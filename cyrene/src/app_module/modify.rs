use std::{fs::File, os::unix::fs::PermissionsExt};

use rune::{ContextError, Module};
#[rune::function]
fn set_exec(path: &str) {
    let file = File::open(path).unwrap();
    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).unwrap();
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate("modify")?;
    m.function_meta(set_exec)?;
    Ok(m)
}
