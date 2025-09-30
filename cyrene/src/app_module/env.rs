use rune::{Any, ContextError, Module};

#[derive(Default, Debug, Any, PartialEq, Eq)]
pub struct CyreneEnv {
    #[rune(get)]
    pub version: String,
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::new();
    m.ty::<CyreneEnv>()?;
    Ok(m)
}
