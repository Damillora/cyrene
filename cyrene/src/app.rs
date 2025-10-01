use std::{path::Path, sync::Arc};

use log::debug;
use rune::{
    Context, Diagnostics, Source, Sources, Vm,
    termcolor::{ColorChoice, StandardStream},
};
use semver::Version;

use crate::{
    app_module::{
        env::{self, CyreneEnv},
        modify, sources, strings, versions,
    },
    errors::CyreneError,
};

pub struct CyreneApp {
    script_vm: Vm,
    plugin_name: String,
}

impl CyreneApp {
    pub fn new(path: &Path) -> Result<Box<Self>, CyreneError> {
        let app_name = path
            .file_stem()
            .ok_or(CyreneError::PluginPathError)?
            .to_string_lossy();

        let mut context = Context::with_default_modules()?;
        context.install(versions::module()?)?;
        context.install(sources::module()?)?;
        context.install(env::module()?)?;
        context.install(modify::module()?)?;
        context.install(strings::module()?)?;
        context.install(rune_modules::http::module(true)?)?;
        context.install(rune_modules::json::module(true)?)?;

        let runtime = Arc::new(context.runtime()?);
        let mut sources = Sources::new();
        sources.insert(Source::from_path(path)?)?;

        let mut diagnostics = Diagnostics::new();

        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources)?;
        }

        let unit = result?;
        let unit = Arc::new(unit);
        let vm = Vm::new(runtime, unit);

        Ok(Box::new(Self {
            plugin_name: String::from(app_name),
            script_vm: vm,
        }))
    }

    pub fn plugin_name(&self) -> String {
        self.plugin_name.clone()
    }

    pub fn get_versions(&mut self) -> Result<Vec<String>, CyreneError> {
        let output = self.script_vm.call(["get_versions"], ())?;
        let output: Vec<String> = rune::from_value(output)?;
        let mut output: Vec<_> = output
            .iter()
            .map(|a| a.trim_start_matches("v"))
            .filter_map(|a| Version::parse(a).ok())
            .collect();
        output.sort_by(|a, b| b.cmp(a));
        let output: Vec<String> = output.iter().map(|a| a.to_string()).collect();

        Ok(output)
    }

    pub fn install_version(
        &mut self,
        installation_dir: &Path,
        version: &str,
    ) -> Result<(), CyreneError> {
        std::env::set_current_dir(installation_dir)?;
        debug!(
            "Installing {} version {} to {}",
            self.plugin_name,
            version,
            installation_dir.to_string_lossy()
        );
        self.script_vm.call(
            ["install_app"],
            (CyreneEnv {
                version: version.into(),
            },),
        )?;
        std::env::set_current_dir(installation_dir)?;
        self.script_vm.call(
            ["post_install"],
            (CyreneEnv {
                version: version.into(),
            },),
        )?;

        Ok(())
    }

    pub fn binaries(&mut self, version: &str) -> Result<Vec<(String, String)>, CyreneError> {
        debug!(
            "Listing binaries of {} version {}",
            self.plugin_name, version
        );
        let result = self.script_vm.call(
            ["binaries"],
            (CyreneEnv {
                version: version.to_string(),
            },),
        )?;
        let output: Vec<(String, String)> = rune::from_value(result)?;

        Ok(output)
    }
}
