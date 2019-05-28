use futures::future;
use hyper;
use hyper::client::Client;
use hyper::header::HeaderValue;
use hyper::rt::Future;
use hyper::HeaderMap;
use hyper::StatusCode;
use hyper::{Body, Request};
use hyper::{Method, Uri};

pub const BYTES_RANGE_TYPE: &'static str = "bytes";
pub const BINARY_CONTENT_TYPE: &'static str = "binary/octet-stream";

#[derive(Debug, PartialEq)]
pub struct FileMetadata {
    // accept_ranges: &'static str,
    // content_type: &'static str,
    content_length: i64,
    etag: Option<String>,
}

pub fn fetch_file_metadata(
    uri: Uri,
) -> impl Future<Item = Option<FileMetadata>, Error = hyper::error::Error> {
    /***********************************
     * TODO:
     * - add TLS connector
     * - handle error if response "accept-ranges" header != BYTE_RANGE_TYPE
     * - handle error if response "content-type" header != BINARY_CONTENT_TYPE
     ***********************************/
    let req = Request::builder()
        .uri(&uri)
        .method(Method::HEAD)
        .body(Body::empty())
        .expect("Failed to build request object");

    Client::new().request(req).and_then(|res| {
        let (status, headers) = (res.status(), res.headers());
        match has_file_metadata(status, headers) {
            false => future::ok(None),
            true => future::ok(Some(parse_file_metadata(headers))),
        }
    })
}

fn has_file_metadata(status: StatusCode, headers: &HeaderMap<HeaderValue>) -> bool {
    status == StatusCode::OK
        && headers.get("accept-ranges").unwrap() == BYTES_RANGE_TYPE
        && headers.get("content-type").unwrap() == BINARY_CONTENT_TYPE
}

fn parse_file_metadata(headers: &HeaderMap<HeaderValue>) -> FileMetadata {
    let content_length = headers
        .get("content-length")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let etag: Option<String> = headers.get("etag").map(|val| {
        let slice = val.to_str().unwrap();
        slice[1..slice.len() - 1].to_string()
    });

    FileMetadata {
        content_length,
        etag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::current_thread::Runtime;

    const SMALL_FILE_URL: &'static str = "http://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";

    #[test]
    fn fetching_file_metadata() {
        let uri = SMALL_FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let future_result = fetch_file_metadata(uri).and_then(move |info| {
            assert_eq!(
                info,
                Some(FileMetadata {
                    content_length: 53143,
                    etag: Some(String::from("ac89ac31a669c13ec4ce037f1203022c"))
                })
            );
            future::ok(())
        });

        rt.block_on(future_result).unwrap();
    }

    #[test]
    fn handling_absent_metadata() {
        let uri = "http://google.com".parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let future_result = fetch_file_metadata(uri).and_then(move |info| {
            assert_eq!(info, None,);
            future::ok(())
        });

        rt.block_on(future_result).unwrap();
    }
}
