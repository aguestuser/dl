use crate::error::DlError;
use crate::https::HttpsClient;
use futures::{Future, Stream};
use hyper;
// use hyper::rt::Future;
use hyper::{Body, Response, Uri};
use std::path::Path;
use tokio_fs::File;
use tokio_io::io;

pub fn download_file(
    client: &HttpsClient,
    uri: Uri,
    target: &'static Path,
) -> Box<Future<Item = (), Error = DlError> + Send> {
    let response_future = client.get(uri).map_err(|err| DlError::Http(err));
    let file_future = File::create(target).map_err(|err| DlError::Io(err));
    Box::new(response_future.join(file_future).and_then(write_to_file))
}

fn write_to_file(args: (Response<Body>, File)) -> impl Future<Item = (), Error = DlError> {
    let (response, file) = args;
    response
        .into_body()
        .map_err(|e| DlError::Http(e))
        .fold(file, |file, chunk| {
            io::write_all(file, chunk)
                .map(|(f, _c)| f)
                .map_err(|e| DlError::Io(e))
        })
        .map(drop)
}

#[cfg(test)]
mod download_tests {
    use super::*;
    use crate::checksum;
    use crate::https;
    use tokio::runtime::Runtime;

    const SMALL_FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
    const SMALL_FILE_MD5_SUM: &'static str = "ac89ac31a669c13ec4ce037f1203022c";

    lazy_static! {
        pub static ref CLIENT: HttpsClient = { https::get_client() };
    }

    #[test]
    fn downloading_file() {
        let uri = SMALL_FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let result =
            download_file(&CLIENT, uri, Path::new("data/foo.pdf"))
                .map(|_| {
                    assert!(
                        checksum::md5sum_check("data/foo.pdf", SMALL_FILE_MD5_SUM).unwrap_or(false)
                    );
                })
                .and_then(|_| {
                    tokio_fs::remove_file("data/foo.pdf").map_err(|err| DlError::Io(err))
                });

        rt.block_on(result).unwrap();
    }
}
