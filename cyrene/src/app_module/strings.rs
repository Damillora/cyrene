use rune::{ContextError, Module};

#[rune::function(instance)]
fn strip_prefix(value: &str, trim: String) -> String {
    value.trim_start_matches(&trim).to_string()
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::new();
    m.function_meta(strip_prefix)?;
    Ok(m)
}
