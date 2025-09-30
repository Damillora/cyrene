use std::fs::File;

use flate2::read::GzDecoder;
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
#[rune::function]
fn from_tar_gz(url: String) {
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().unwrap();
    let tar_gz = GzDecoder::new(res);
    let mut tar = Archive::new(tar_gz);
    tar.unpack(".").unwrap();
}
#[rune::function]
fn from_file(url: String) {
    let client = reqwest::blocking::Client::new();
    let mut res = client.get(&url).send().unwrap();
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap()
        .to_string();
    let mut file = File::create(target_filename).unwrap();
    std::io::copy(&mut res, &mut file).unwrap();
}
#[rune::function]
fn from_file_dest(url: String, dest: String) {
    let client = reqwest::blocking::Client::new();
    let mut res = client.get(&url).send().unwrap();
    let mut file = File::create(dest).unwrap();
    std::io::copy(&mut res, &mut file).unwrap();
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate("sources")?;
    m.function_meta(from_tar_xz)?;
    m.function_meta(from_tar_gz)?;
    m.function_meta(from_file)?;
    m.function_meta(from_file_dest)?;
    Ok(m)
}
