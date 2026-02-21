use std::{
    collections::HashMap,
    io::{self},
    path::Path,
};

use async_compression::futures::{bufread::GzipDecoder, bufread::XzDecoder};
use async_tar::Archive;
use futures::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use tempfile::tempfile;
use text_template::Template;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use zip::ZipArchive;

use crate::{app::AppSources, errors::CyreneError};

fn new_progress_bar(filename: &str, len: u64) -> ProgressBar {
    ProgressBar::new(len)
        .with_style(
            ProgressStyle::with_template(
                "{msg:.white.bold} {wide_bar:.219} {percent:>3.219.bold}% [{bytes:>10.white}/{total_bytes:.219.bold}]",
            )
            .unwrap(),
        )
        .with_message(filename.to_string())
}

async fn from_tar_xz(url: &str, dest: &Path) -> Result<(), CyreneError> {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| CyreneError::Download(url.to_string(), e))?;
    let len = res.content_length().unwrap();

    let reader = res
        .bytes_stream()
        .map_err(io::Error::other)
        .into_async_read()
        .compat();
    let pb = new_progress_bar(&target_filename, len);
    let reader = pb.wrap_async_read(reader);

    let tar_xz = XzDecoder::new(reader.compat());
    let tar = Archive::new(tar_xz);
    tar.unpack(dest).await.unwrap();

    Ok(())
}

async fn from_tar_gz(url: &str, dest: &Path) -> Result<(), CyreneError> {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| CyreneError::Download(url.to_string(), e))?;
    let len = res.content_length().unwrap();

    let reader = res
        .bytes_stream()
        .map_err(io::Error::other)
        .into_async_read()
        .compat();
    debug!("len: {}", len);
    let pb = new_progress_bar(&target_filename, len);
    let reader = pb.wrap_async_read(reader);

    let tar_gz = GzipDecoder::new(reader.compat());
    let tar = Archive::new(tar_gz);
    tar.unpack(dest).await.unwrap();

    Ok(())
}

async fn from_zip(url: &str, dest: &Path) -> Result<(), CyreneError> {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| CyreneError::Download(url.to_string(), e))?;
    let len = res.content_length().unwrap();

    let reader = res
        .bytes_stream()
        .map_err(io::Error::other)
        .into_async_read()
        .compat();
    let pb = new_progress_bar(&target_filename, len);
    let mut reader = pb.wrap_async_read(reader);

    let mut file = tokio::fs::File::from_std(tempfile().unwrap());

    tokio::io::copy(&mut reader, &mut file)
        .await
        .map_err(|e| CyreneError::DownloadWrite(url.to_string(), e))?;

    let mut zip_file = ZipArchive::new(file.into_std().await).unwrap();
    zip_file.extract(dest).unwrap();

    Ok(())
}

async fn from_file(url: &str, dest: &Path) -> Result<(), CyreneError> {
    let target_filename = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap()
        .to_string();
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| CyreneError::Download(url.to_string(), e))?;
    let len = res.content_length().unwrap();

    let reader = res
        .bytes_stream()
        .map_err(io::Error::other)
        .into_async_read()
        .compat();
    let pb = new_progress_bar(&target_filename, len);
    let mut reader = pb.wrap_async_read(reader);

    let mut target_file = dest.to_path_buf();
    target_file.push(&target_filename);

    let mut file = tokio::fs::File::create(&target_file)
        .await
        .map_err(|e| CyreneError::DownloadWrite(target_file.to_string_lossy().to_string(), e))?;

    tokio::io::copy(&mut reader, &mut file)
        .await
        .map_err(|e| CyreneError::DownloadWrite(url.to_string(), e))?;

    Ok(())
}

pub async fn process_source(
    source: &AppSources,
    version: &str,
    dest: &Path,
) -> Result<(), CyreneError> {
    let mut values = HashMap::new();
    values.insert("version", version);
    match source {
        AppSources::TarXz { url } => {
            let tmpl = Template::from(url.as_str());
            let url = tmpl.fill_in(&values);
            from_tar_xz(&url.to_string(), dest).await
        }
        AppSources::TarGz { url } => {
            let tmpl = Template::from(url.as_str());
            let url = tmpl.fill_in(&values);
            from_tar_gz(&url.to_string(), dest).await
        }
        AppSources::Zip { url } => {
            let tmpl = Template::from(url.as_str());
            let url = tmpl.fill_in(&values);
            from_zip(&url.to_string(), dest).await
        }
        AppSources::File { url } => {
            let tmpl = Template::from(url.as_str());
            let url = tmpl.fill_in(&values);
            from_file(&url.to_string(), dest).await
        }
    }
}
