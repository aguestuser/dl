use crate::error::DlError;
use futures::{Future, IntoFuture};
use hex;
use md5::{Digest, Md5};
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct HashChecker {
    pub path: OsString,
    pub etag: Option<String>,
}

impl HashChecker {
    pub fn check(self) -> impl Future<Item = (OsString, bool), Error = DlError> {
        let p = self.path.clone();
        match self.etag {
            None => Err(DlError::EtagAbsent),
            Some(etag) => md5sum_check(&PathBuf::from(p.clone()), &etag).map(|valid| (p, valid)),
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
    let mut f = File::open(path)?;
    f.read_to_end(&mut buffer)?;

    let mut hasher = Md5::new();
    hasher.input(buffer);
    Ok(hasher.result().to_vec())
}

#[cfg(test)]
mod checksum_tests {
    use super::*;
    use std::error::Error;
    use tokio::runtime::Runtime;

    #[test]
    fn taking_m5sum() {
        assert_eq!(
            md5sum(Path::new("data/foo.txt")).unwrap(),
            hex::decode("d3b07384d113edec49eaa6238ad5ff00").unwrap(),
        );
    }

    #[test]
    fn checking_md5sum() {
        assert_eq!(
            md5sum_check(
                Path::new("data/foo.txt"),
                "d3b07384d113edec49eaa6238ad5ff00"
            )
            .unwrap(),
            true,
        )
    }

    #[test]
    fn running_hash_checker_with_etag() {
        let hc = HashChecker {
            path: OsString::from("data/foo.txt"),
            etag: Some(String::from("d3b07384d113edec49eaa6238ad5ff00")),
        };

        let res = hc.check().map(|(_, valid)| {
            assert_eq!(valid, true);
        });

        Runtime::new().unwrap().block_on(res).unwrap();
    }

    #[test]
    fn running_hash_checker_without_etag() {
        let hc = HashChecker {
            path: OsString::from("data/foo.txt"),
            etag: None,
        };

        let res = hc.check().map_err(|err| {
            assert_eq!(err.description(), DlError::EtagAbsent.description());
        });

        Runtime::new().unwrap().block_on(res).err();
    }
}
