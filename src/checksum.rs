use crate::error::DlError;
use hex;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;

pub fn md5sum(path: &str) -> Result<Vec<u8>, DlError> {
    let mut buffer = Vec::new();
    let mut f = File::open(path)?;
    f.read_to_end(&mut buffer)?;

    let mut hasher = Md5::new();
    hasher.input(buffer);
    Ok(hasher.result().to_vec())
}

pub fn md5sum_check(path: &str, sum_hex: &str) -> Result<bool, DlError> {
    md5sum(path)
        .iter()
        .zip(hex::decode(sum_hex).iter())
        .map(|(sum, actual_sum)| sum == actual_sum)
        .next()
        .ok_or(DlError::Checksum)
}

#[cfg(test)]
mod checksum_tests {
    use super::*;

    #[test]
    fn taking_m5sum() {
        assert_eq!(
            md5sum("data/foo.txt").unwrap(),
            hex::decode("d3b07384d113edec49eaa6238ad5ff00").unwrap(),
        );
    }

    #[test]
    fn checking_md5sum() {
        assert_eq!(
            md5sum_check("data/foo.txt", "d3b07384d113edec49eaa6238ad5ff00").unwrap(),
            true,
        )
    }
}
