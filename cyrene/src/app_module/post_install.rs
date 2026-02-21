use std::{collections::HashMap, fs::File, os::unix::fs::PermissionsExt, path::Path};

use text_template::Template;

use crate::{app::AppPostInstallCommands, errors::CyreneError};

fn set_exec(path: &str, dest: &Path) -> Result<(), CyreneError> {
    let mut target_file = dest.to_path_buf();
    target_file.push(path);

    let file = File::open(target_file).unwrap();
    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).unwrap();

    Ok(())
}

pub async fn process_post_install(
    command: &AppPostInstallCommands,
    version: &str,
    dest: &Path,
) -> Result<(), CyreneError> {
    let mut values = HashMap::new();
    values.insert("version", version);
    match command {
        AppPostInstallCommands::SetExec { path } => {
            let path_tmpl = Template::from(path.as_str());
            let path = path_tmpl.fill_in(&values).to_string();
            set_exec(&path, dest)?
        }
    };

    Ok(())
}
