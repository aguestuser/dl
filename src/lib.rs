use futures::future;
use hyper;
use hyper::client::Client;
use hyper::rt::Future;
use hyper::{Body, Request, Response};
use hyper::{Method, Uri};

pub const BYTES_RANGE_TYPE: &'static str = "bytes";
pub const BINARY_CONTENT_TYPE: &'static str = "binary/octet-stream";

#[derive(Debug, PartialEq)]
pub struct Info {
    // accept_ranges: &'static str,
    // content_type: &'static str,
    content_length: i64,
    etag: Option<String>,
}

pub fn get_info(uri: Uri) -> impl Future<Item = Response<Body>, Error = ()> {
    /***********************************
     * TODO:
     * - add TLS connector
     * - short circuit if response "accept-ranges" header != BYTE_RANGE_TYPE
     * - short circuit if response "content-type" header != BINARY_CONTENT_TYPE
     ***********************************/
    let req = Request::builder()
        .uri(&uri)
        .method(Method::HEAD)
        .body(Body::empty())
        .expect("Failed to build request object")

    Client::new()
        .request(req)
        .and_then(|res| future::ok(res))
        .map_err(|err| eprintln!("Error: {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::current_thread::Runtime;

    const SMALL_FILE_URL: &'static str = "http://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";

    #[test]
    fn getting_file_info() {
        let uri = SMALL_FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let future_result = get_info(uri).and_then(move |resp| {
            let (status, headers) = (resp.status(), resp.headers());

            // assert the response is well-formed
            assert_eq!(status, hyper::StatusCode::OK);
            assert_eq!(headers.get("accept-ranges").unwrap(), BYTES_RANGE_TYPE);
            assert_eq!(headers.get("content-type").unwrap(), BINARY_CONTENT_TYPE);

            // assert the response contains correct data bout resume
            assert_eq!(headers.get("content-length").unwrap(), &"53143");
            assert_eq!(
                headers.get("etag").unwrap(),
                &"\"ac89ac31a669c13ec4ce037f1203022c\""
            );
            future::ok(())
        });

        rt.block_on(future_result).unwrap();
    }
}
