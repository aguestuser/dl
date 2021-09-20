use std::path::PathBuf;

use futures::future::IntoFuture;
use hyper;
use hyper::header::HeaderValue;
use hyper::rt::Future;
use hyper::HeaderMap;
use hyper::StatusCode;
use hyper::{Body, Request};
use hyper::{Method, Uri};

use crate::error::DlError;
use crate::file::FileDownloader;
use crate::https::{self, HttpsClient};
use crate::Config;
use crate::DEFAULT_PARALLELISM;

pub const BYTES_RANGE_TYPE: &'static str = "bytes";
pub const BINARY_CONTENT_TYPE: &'static str = "binary/octet-stream";

#[derive(Debug, PartialEq)]
pub struct Metadata {
    pub file_size: u64,
    pub etag: Option<String>,
}

#[derive(Debug)]
pub struct MetadataDownloader {
    pub client: HttpsClient,
    pub uri: Uri,
    pub path: PathBuf,
    pub parallelism: usize,
}

impl MetadataDownloader {
    /// constructs a `MetadataDownloader` from a `Config` struct
    pub fn from_config(cfg: Config) -> MetadataDownloader {
        Self {
            client: https::get_client(cfg.parallelism),
            uri: cfg.uri,
            path: cfg.path,
            parallelism: cfg.parallelism,
        }
    }

    /// Tries several strategies to return file metadata and returns an error if not possible
    pub fn fetch(self) -> impl Future<Item = FileDownloader, Error = DlError> {
        // TODO: write a `fetch` that tries several strategies
        self.fetch_head()
    }

    /// Issues a HEAD request to the downloader's `uri`.
    ///
    /// Inspects the response to determine:
    /// - whether the uri supports range requests or not
    /// - the size of the file, and (optionally) its etag
    ///
    /// **Happy path:** Resolves future with `Metadata` struct
    ///
    /// **Sad path:** Resolves future with `Error` indicating whether:
    /// - request or header parsing failed
    /// - metadata headers not present
    pub fn fetch_head(self) -> impl Future<Item = FileDownloader, Error = DlError> {
        let req = Request::builder()
            .uri(&self.uri)
            .method(Method::HEAD)
            .body(Body::empty())
            .expect("Failed to build request object");

        self.client
            .request(req)
            .map_err(DlError::Hyper)
            .and_then(|res| {
                let (status, headers) = (res.status(), res.headers());
                is_success(status)
                    .and_then(|_| have_file_metadata(headers))
                    .and_then(|_| parse_file_metadata(headers))
                    .map(|md| FileDownloader::from_metadata(self, md))
                    .into_future()
            })
    }
}

fn is_success(status: StatusCode) -> Result<(), DlError> {
    match status.is_success() || status.is_redirection() {
        true => Ok(()),
        false => Err(DlError::RequestFailed(status.as_u16())),
    }
}

fn have_file_metadata(headers: &HeaderMap<HeaderValue>) -> Result<(), DlError> {
    match headers.get("accept-ranges") == Some(&HeaderValue::from_static(BYTES_RANGE_TYPE))
        // && headers.get("content-type") == Some(&HeaderValue::from_static(BINARY_CONTENT_TYPE))
    {
        true => Ok(()),
        false => Err(DlError::RangeMetadataAbsent),
    }
}

fn parse_file_metadata(headers: &HeaderMap<HeaderValue>) -> Result<Metadata, DlError> {
    let etag: Option<String> = parse_etag(headers);
    parse_length(headers).map(|file_size| Metadata { file_size, etag })
}

fn parse_length(headers: &HeaderMap<HeaderValue>) -> Result<u64, DlError> {
    headers
        .get("content-length")
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or(DlError::ParseContentLength)
}

fn parse_etag(headers: &HeaderMap<HeaderValue>) -> Option<String> {
    headers
        .get("etag")
        .map(|val| val.to_str())
        .and_then(|maybe_str| maybe_str.ok().map(|s| s[1..s.len() - 1].to_string()))
    // remove escaped quotes
}

#[cfg(test)]
mod metadata_tests {
    use std::error::Error;

    use tokio::runtime::Runtime;

    use crate::https;
    use crate::DEFAULT_PARALLELISM;

    use super::*;

    const SMALL_FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";

    #[test]
    fn fetching_file_metadata() {
        let mdd = MetadataDownloader {
            client: https::get_client(*DEFAULT_PARALLELISM),
            uri: SMALL_FILE_URL.parse::<Uri>().unwrap(),
            path: PathBuf::from("data/foo_meta.pdf"),
            parallelism: *DEFAULT_PARALLELISM,
        };

        let fd = Runtime::new().unwrap().block_on(mdd.fetch()).unwrap();

        assert_eq!(fd.file_size, 53143);
        assert_eq!(
            fd.etag,
            Some(String::from("ac89ac31a669c13ec4ce037f1203022c"))
        );
    }

    #[test]
    fn handling_absent_file_metadata() {
        let mdd = MetadataDownloader {
            client: https::get_client(*DEFAULT_PARALLELISM),
            uri: "https://google.com".parse::<Uri>().unwrap(),
            path: PathBuf::from("data/foo_meta.pdf"),
            parallelism: *DEFAULT_PARALLELISM,
        };

        let future_result = mdd.fetch();
        let err = Runtime::new()
            .unwrap()
            .block_on(future_result)
            .err()
            .unwrap();

        assert_eq!(
            err.description(),
            DlError::RangeMetadataAbsent.description()
        );
    }
}
