use rune::{ContextError, Module};
use tar::Archive;
use xz::read::XzDecoder;

#[rune::function]
fn from_tar_xz(url: String) {
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().unwrap();
    let tar_xz = XzDecoder::new(res);
    let mut tar = Archive::new(tar_xz);
    tar.unpack(".").unwrap();
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate("sources")?;
    m.function_meta(from_tar_xz)?;
    Ok(m)
}
