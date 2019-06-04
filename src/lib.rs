#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use crate::metadata::MetadataDownloader;
use error::DlError;
use futures::Future;
use hyper::Uri;
use std::path::PathBuf;

pub mod checksum;
pub mod error;
pub mod file;
pub mod https;
pub mod metadata;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub uri: Uri,
    pub path: PathBuf,
}

// these macros are weird but we need them b/c we cannot concat constant string constants in rust
// nor can we pass `String` objects (which we can concatenate) to `Err`
// see: https://github.com/rust-lang/rust/issues/31383
macro_rules! usage {
    () => {
        "> Correct usage: dl <valid_url> <output_path>"
    };
}

macro_rules! insufficient_args {
    () => {
        concat!("> Error: please provide 2 arguments", "\n", usage!())
    };
}
macro_rules! invalid_uri {
    () => {
        concat!("> Error: invalid uri", "\n", usage!())
    };
}

impl Config {
    pub fn new(args: Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err(insufficient_args!());
        }

        let uri = match args[1].parse::<Uri>() {
            Ok(u) => u,
            _ => return Err(invalid_uri!()),
        };

        let path = PathBuf::from(&args[2]);

        Ok({ Config { uri, path } })
    }
}

pub fn run(cfg: Config) -> impl Future<Item = (), Error = DlError> {
    // TODO: use logger instead of println (to clean up test output)
    println!("> fetching file metadata...");
    MetadataDownloader::from_config(cfg)
        .fetch()
        .and_then(move |file_downloader| {
            println!(
                "> ...found metadata. file size: {}, etag: {}",
                &file_downloader.file_size,
                &file_downloader.etag.clone().unwrap_or(String::from("N/A")),
            );
            println!("> downloading file...");
            file_downloader.fetch()
        })
        .and_then(move |hash_checker| {
            println!("> ...file downloaded!");
            println!(
                "\n>>>>> file ready at: {} <<<<<\n",
                &hash_checker.path.to_str().unwrap()
            );
            println!("> verifying etag (if present)...");
            hash_checker.check().map(move |valid| {
                match valid {
                    true => println!("> ...hashes match!"),
                    false => println!("> ...hashes do not match. :("),
                };
            })
        })
        .map(|_| ())
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    use crate::checksum::md5sum_check;
    use std::error::Error;
    use tokio::runtime::Runtime;

    #[test]
    fn parsing_valid_cli_args() {
        assert_eq!(
            Config::new(vec![
                String::from("dl"),
                String::from("https://foo.com"),
                String::from("bar/baz")
            ])
            .unwrap(),
            Config {
                uri: Uri::from_static("https://foo.com"),
                path: PathBuf::from("bar/baz")
            }
        )
    }

    #[test]
    fn parsing_empty_cli_args() {
        assert_eq!(Config::new(vec![]).err().unwrap(), insufficient_args!());
    }

    #[test]
    fn parsing_invalid_cli_args() {
        assert_eq!(
            Config::new(vec![
                String::from("dl"),
                String::from("foo bar"),
                String::from("bar/baz")
            ])
            .err()
            .unwrap(),
            invalid_uri!()
        )
    }

    #[test]
    fn running_the_app_against_happy_path() {
        let path = PathBuf::from("data/happy.pdf");
        let cfg = Config {
            uri: "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf".parse::<Uri>().unwrap(),
            path: path.clone(),
        };

        Runtime::new().unwrap().block_on(run(cfg)).unwrap();
        assert!(&path.exists());
        assert!(md5sum_check(&path, "ac89ac31a669c13ec4ce037f1203022c").unwrap());

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn running_the_app_against_no_range_link() {
        let path = PathBuf::from("whack");
        let cfg = Config {
            uri: "https://google.com".parse::<Uri>().unwrap(),
            path: path.clone(),
        };

        let err = Runtime::new().unwrap().block_on(run(cfg)).err().unwrap();
        assert!(!&path.exists());
        assert_eq!(
            err.description(),
            DlError::RangeMetadataAbsent.description()
        );
    }

    #[test]
    fn running_the_app_against_no_etag_link() {
        let path = PathBuf::from("data/logo.png");
        let cfg = Config {
            uri: "https://littlesis.org/assets/lilsis-logo-trans-200-74169fd94db9637c31388ad2060b48720f94450b40c45c23a3889cf480f02c52.png".parse::<Uri>().unwrap(),
            path: path.clone(),
        };

        let err = Runtime::new().unwrap().block_on(run(cfg)).err().unwrap();
        assert!(&path.exists());
        assert_eq!(err.description(), DlError::EtagAbsent.description());

        std::fs::remove_file(&path).unwrap();
    }

}
