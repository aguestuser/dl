#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use crate::metadata::MetadataDownloader;
use error::DlError;
use futures::Future;
use hyper::Uri;
use std::ffi::OsString;
use std::path::PathBuf;

pub mod checksum;
pub mod error;
pub mod file;
pub mod https;
pub mod metadata;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub uri: Uri,
    pub path: OsString,
}

impl Config {
    pub fn new(args: Vec<String>) -> Result<Config, &'static str> {
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

        Ok({ Config { uri, path } })
    }
}

pub fn run(cfg: Config) -> impl Future<Item = (), Error = DlError> {
    println!("> fetching file metadata...");
    MetadataDownloader::from_config(cfg)
        .fetch()
        .and_then(move |file_downloader| {
            println!(
                "> found metadata. file size: {}, etag: {:?}",
                &file_downloader.file_size, &file_downloader.etag,
            );
            println!("> downloading file...");
            file_downloader.fetch()
        })
        .and_then(move |hash_checker| {
            println!("> verifying etag (if present)...");
            hash_checker.check().map(move |(path, valid)| {
                match valid {
                    true => println!("> hashes match!"),
                    false => println!("> hashes do not match. :("),
                };
                println!("> your file is ready at: {:?}", path);
            })
        })
        .map(|_| ())
}
