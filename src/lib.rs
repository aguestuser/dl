#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use error::DlError;
use futures::{future, Future};
use hyper::Uri;
use std::path::PathBuf;

pub mod checksum;
pub mod download;
pub mod error;
pub mod https;
pub mod metadata;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub uri: Uri,
    pub path: PathBuf,
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
            Some(_) => maybe_path,
            None => return Err("Invalid file path."),
        };

        Ok({ Config { uri, path } })
    }
}

pub fn run(Config { path, uri }: Config) -> Box<impl Future<Item = (), Error = DlError>> {
    // try to fetch metadata
    // if found fetch file
    // if downloaded and etag avail, check etag
    // if all succeeds, print path to open file and exit

    Box::new(
        download::fetch_par(uri.clone(), path.clone(), 40)
            .and_then(move |_| download::fetch_par(uri.clone(), path.clone(), 40))
            .and_then(move |_| future::ok(())),
    )
}
