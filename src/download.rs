use crate::error::DlError;
use crate::https::HttpsClient;
use futures::{Future, Stream};
use hyper;
// use hyper::rt::Future;
use hyper::{Body, Response, Uri};
use std::path::Path;
use tokio_fs::File;
use tokio_io::io;

pub fn download_file<P: AsRef<Path> + Send + 'static>(
    client: &HttpsClient,
    uri: Uri,
    target: P,
) -> Box<Future<Item = usize, Error = DlError> + Send> {
    // TODO: consider moving Uri parsing logic inside this function
    let response_future = client.get(uri).map_err(|err| DlError::Http(err));
    let file_future = File::create(target).map_err(|err| DlError::Io(err));
    Box::new(response_future.join(file_future).and_then(write_to_file))
}

fn write_to_file(
    (response, file): (Response<Body>, File),
) -> impl Future<Item = usize, Error = DlError> {
    response
        .into_body()
        .map_err(DlError::Http)
        .fold((file, 0), |(file, bytes_written), chunk| {
            io::write_all(file, chunk)
                .map(move |(f, c)| (f, bytes_written + c.len()))
                .map_err(DlError::Io)
        })
        .map(|(f, bytes_written)| {
            drop(f);
            bytes_written
        })
}

#[cfg(test)]
mod download_tests {
    use super::*;
    use crate::checksum;
    use crate::https;
    use tokio::runtime::Runtime;

    const SMALL_FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
    const SMALL_FILE_MD5_SUM: &'static str = "ac89ac31a669c13ec4ce037f1203022c";
    const TARGET_PATH: &'static str = "data/foo.pdf";

    lazy_static! {
        pub static ref CLIENT: HttpsClient = { https::get_client() };
    }

    #[test]
    fn downloading_file() {
        let uri = SMALL_FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let result = download_file(&CLIENT, uri, TARGET_PATH)
            .map(|bytes_written| {
                assert_eq!(bytes_written, 53143);
                assert!(checksum::md5sum_check(TARGET_PATH, SMALL_FILE_MD5_SUM).unwrap_or(false));
            })
            .and_then(|_| tokio_fs::remove_file(TARGET_PATH).map_err(|err| DlError::Io(err)));

        rt.block_on(result).unwrap();
    }
}
