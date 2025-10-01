use std::{
    fs::File,
    io::{self, Read},
};

use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use rune::{ContextError, Module};
use tar::Archive;
use xz::read::XzDecoder;
struct UploadProgress<R> {
    inner: R,
    total: u64,
    bytes_read: u64,
    progress_bar: ProgressBar,
}
impl<R: Read> UploadProgress<R> {
    fn new(read: R, filename: &str, len: u64) -> Self {
        Self {
            inner: read,
            bytes_read: 0,
            total: len,
            progress_bar: ProgressBar::new(len)
                .with_style(
                    ProgressStyle::with_template(
                        "{msg:.white.bold} {wide_bar:.219} {percent:>3.219.bold}% [{bytes:>10.white}/{total_bytes:.219.bold}]",
                    )
                    .unwrap(),
                )
                .with_message(filename.to_string()),
        }
    }
}
impl<R: Read> Read for UploadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).inspect(|n| {
            self.bytes_read += *n as u64;
            self.progress_bar.inc(*n as u64);
            if self.bytes_read == self.total {
                self.progress_bar.finish();
            }
        })
    }
}

#[rune::function]
fn from_tar_xz(url: &str) {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().unwrap();
    let len = res.content_length();
    let res: Box<dyn Read> = if let Some(len) = len {
        Box::new(UploadProgress::new(res, &target_filename, len))
    } else {
        Box::new(res)
    };
    let tar_xz = XzDecoder::new(res);
    let mut tar = Archive::new(tar_xz);
    tar.unpack(".").unwrap();
}
#[rune::function]
fn from_tar_gz(url: &str) {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().unwrap();
    let len = res.content_length();
    let res: Box<dyn Read> = if let Some(len) = len {
        Box::new(UploadProgress::new(res, &target_filename, len))
    } else {
        Box::new(res)
    };
    let tar_gz = GzDecoder::new(res);
    let mut tar = Archive::new(tar_gz);
    tar.unpack(".").unwrap();
}
#[rune::function]
fn from_file(url: &str) {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().unwrap();
    let len = res.content_length();
    let mut res: Box<dyn Read> = if let Some(len) = len {
        Box::new(UploadProgress::new(res, &target_filename, len))
    } else {
        Box::new(res)
    };
    let mut file = File::create(target_filename).unwrap();
    std::io::copy(&mut res, &mut file).unwrap();
}
#[rune::function]
fn from_file_dest(url: &str, dest: &str) {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().unwrap();
    let len = res.content_length();
    let mut res: Box<dyn Read> = if let Some(len) = len {
        Box::new(UploadProgress::new(res, &target_filename, len))
    } else {
        Box::new(res)
    };
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
