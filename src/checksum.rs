use crate::error::DlError;
use futures::{Future, IntoFuture};
use hex;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct HashChecker {
    pub path: PathBuf,
    pub etag: Option<String>,
}

impl HashChecker {
    pub fn check(self) -> impl Future<Item = bool, Error = DlError> {
        let p = self.path.clone();
        match self.etag {
            None => Err(DlError::EtagAbsent),
            Some(etag) => md5sum_check(&PathBuf::from(p.clone()), &etag),
        }
        .into_future()
    }
}

pub fn md5sum_check(path: &Path, sum_hex: &str) -> Result<bool, DlError> {
    md5sum(path)
        .iter()
        .zip(hex::decode(sum_hex).iter())
        .map(|(sum, actual_sum)| sum == actual_sum)
        .next()
        .ok_or(DlError::Checksum)
}

// TODO: this should take a PathBuf
pub fn md5sum(path: &Path) -> Result<Vec<u8>, DlError> {
    let mut buffer = Vec::new();
    let mut hasher = Md5::new();

    File::open(path)
        .and_then(|mut f| f.read_to_end(&mut buffer))
        .map_err(DlError::Io)
        .map(|_| hasher.input(buffer))
        .map(|_| hasher.result().to_vec())
}

#[cfg(test)]
mod checksum_tests {
    use super::*;
    use std::error::Error;
    use tokio::runtime::Runtime;

    #[test]
    fn taking_m5sum() {
        assert_eq!(
            md5sum(&PathBuf::from("data/foo.txt")).unwrap(),
            hex::decode("d3b07384d113edec49eaa6238ad5ff00").unwrap(),
        );
    }

    #[test]
    fn checking_md5sum() {
        assert_eq!(
            md5sum_check(
                &PathBuf::from("data/foo.txt"),
                "d3b07384d113edec49eaa6238ad5ff00"
            )
            .unwrap(),
            true,
        )
    }

    #[test]
    fn running_hash_checker_with_etag() {
        let hc = HashChecker {
            path: PathBuf::from("data/foo.txt"),
            etag: Some(String::from("d3b07384d113edec49eaa6238ad5ff00")),
        };
        let valid = Runtime::new().unwrap().block_on(hc.check()).unwrap();
        assert_eq!(valid, true);
    }

    #[test]
    fn running_hash_checker_without_etag() {
        let hc = HashChecker {
            path: PathBuf::from("data/foo.txt"),
            etag: None,
        };
        let err = Runtime::new().unwrap().block_on(hc.check()).err().unwrap();
        assert_eq!(err.description(), DlError::EtagAbsent.description());
    }
}
