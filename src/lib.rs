#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use crate::file::FileDownloader;
use crate::metadata::MetadataDownloader;
use error::DlError;
use futures::{future, Future};
use hyper::Uri;
use std::ffi::OsString;
use std::path::PathBuf;

pub mod checksum;
pub mod error;
pub mod file;
pub mod https;
pub mod metadata;

pub enum Downloader {
    Config(DlConfig),
    Metadata(MetadataDownloader),
    File(FileDownloader),
}

#[derive(Debug, PartialEq)]
pub struct DlConfig {
    pub uri: Uri,
    pub path: OsString,
}

impl DlConfig {
    pub fn new(args: Vec<String>) -> Result<DlConfig, &'static str> {
        if args.len() < 3 {
            return Err("Error: please provide 2 arguments. \nCorrect usage: \n    dl <valid_url> <output_path>");
        }

        let uri = match args[1].parse::<Uri>() {
            Ok(u) => u,
            _ => return Err("Invalid uri."),
        };

        let maybe_path = PathBuf::from(&args[2]);
        let path = match maybe_path.file_name() {
            Some(_) => maybe_path.into_os_string(),
            None => return Err("Invalid file path."),
        };

        Ok({ DlConfig { uri, path } })
    }
}

pub fn run(cfg: DlConfig) -> impl Future<Item = (), Error = DlError> {
    // try to fetch metadata
    // if found fetch file
    // if downloaded and etag avail, check etag
    // if all succeeds, print path to open file and exit

    MetadataDownloader::from_config(cfg)
        .fetch()
        .and_then(move |file_downloader| file_downloader.fetch())
        .map(|_| ())
}
