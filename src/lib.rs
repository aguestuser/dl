use hyper;
use hyper::client::Client;
use hyper::error::Error as HyperError;
use hyper::header::HeaderValue;
use hyper::rt::Future;
use hyper::HeaderMap;
use hyper::StatusCode;
use hyper::{Body, Request};
use hyper::{Method, Uri};
use std::error::Error;
use std::fmt;

pub const BYTES_RANGE_TYPE: &'static str = "bytes";
pub const BINARY_CONTENT_TYPE: &'static str = "binary/octet-stream";

#[derive(Debug)]
pub enum DlError {
    ParseContentLength,
    ValidFileMetadata,
}

impl fmt::Display for DlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DlError::ValidFileMetadata => write!(f, "File does not have valid metadata",),
            DlError::ParseContentLength => write!(f, "Failed to parse content length header",),
        }
    }
}

impl Error for DlError {
    fn description(&self) -> &str {
        match *self {
            DlError::ValidFileMetadata => "File does not have valid metadata",
            DlError::ParseContentLength => "Failed to parse content length header",
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FileMetadata {
    content_length: i64,
    etag: Option<String>,
}

pub fn fetch_file_metadata(
    uri: Uri,
) -> impl Future<Item = Result<FileMetadata, DlError>, Error = HyperError> {
    /***********************************
     * TODO: add TLS connector
     ***********************************/
    let req = Request::builder()
        .uri(&uri)
        .method(Method::HEAD)
        .body(Body::empty())
        .expect("Failed to build request object");

    Client::new().request(req).map(|res| {
        let (status, headers) = (res.status(), res.headers());
        has_file_metadata(status, headers).and_then(|_| parse_file_metadata(headers))
    })
}

fn has_file_metadata(status: StatusCode, headers: &HeaderMap<HeaderValue>) -> Result<(), DlError> {
    match status == StatusCode::OK
        && headers.get("accept-ranges").unwrap() == BYTES_RANGE_TYPE
        && headers.get("content-type").unwrap() == BINARY_CONTENT_TYPE
    {
        true => Ok(()),
        false => Err(DlError::ValidFileMetadata),
    }
}

fn parse_file_metadata(headers: &HeaderMap<HeaderValue>) -> Result<FileMetadata, DlError> {
    let etag: Option<String> = parse_etag(headers);
    parse_content_length(headers).map(|content_length| FileMetadata {
        content_length,
        etag,
    })
}

fn parse_content_length(headers: &HeaderMap<HeaderValue>) -> Result<i64, DlError> {
    headers
        .get("content-length")
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or(DlError::ParseContentLength)
}

fn parse_etag(headers: &HeaderMap<HeaderValue>) -> Option<String> {
    headers
        .get("etag")
        .map(|val| val.to_str())
        .and_then(|maybe_str| maybe_str.ok().map(|s| s[1..s.len() - 1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future;
    use tokio::runtime::current_thread::Runtime;

    const SMALL_FILE_URL: &'static str = "http://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";

    #[test]
    fn fetching_file_metadata() {
        let uri = SMALL_FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let future_result = fetch_file_metadata(uri).and_then(move |res| {
            assert_eq!(
                res.unwrap(),
                FileMetadata {
                    content_length: 53143,
                    etag: Some(String::from("ac89ac31a669c13ec4ce037f1203022c"))
                }
            );
            future::ok(())
        });

        rt.block_on(future_result).unwrap();
    }

    #[test]
    fn handling_absent_file_metadata() {
        let uri = "http://google.com".parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let future_result = fetch_file_metadata(uri).and_then(move |res| {
            assert_eq!(
                res.err().unwrap().description(),
                DlError::ValidFileMetadata.description()
            );
            future::ok(())
        });

        rt.block_on(future_result).unwrap();
    }
}
