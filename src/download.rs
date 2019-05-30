use crate::error::DlError;
use crate::https::HttpsClient;
use futures::{future, stream, Future, Stream};
use hyper;
use tokio_io::AsyncWrite;
// use hyper::rt::Future;
use hyper::{Body, Response, Uri};
use std::path::Path;
use tokio_fs::File;
use tokio_io::io;

// the trait bound describing a typesafe static string to be interpreted as a path by tokio_io
// (it's a mouthful, so we alias it)
pub trait ThreadsafePath: AsRef<Path> + Send + 'static {}
impl<T: AsRef<Path> + Send + 'static> ThreadsafePath for T {}

// pub fn download_file<P: ThreadsafePath>(
//     client: &HttpsClient,
//     uri: Uri,
//     _size: usize,
//     target: P,
// ) -> Box<Future<Item = usize, Error = DlError> + Send> {
//     // TODO: consider moving Uri parsing logic inside this function
//     let response_future = client.get(uri).map_err(|err| DlError::Http(err));
//     let file_future = File::create(target).map_err(|err| DlError::Io(err));
//     Box::new(response_future.join(file_future).and_then(write_to_file))
// }

// fn write_to_file(
//     (response, file): (Response<Body>, File),
// ) -> impl Future<Item = usize, Error = DlError> {
//     response
//         .into_body()
//         .map_err(DlError::Http)
//         .fold((file, 0), |(file, bytes_written), chunk| {
//             io::write_all(file, chunk)
//                 .map(move |(f, c)| (f, bytes_written + c.len()))
//                 .map_err(DlError::Io)
//         })
//         .map(|(f, bytes_written)| {
//             drop(f);
//             bytes_written
//         })
// }

/// given a http `client`, the `uri` for a file, the file's `size` (in bytes) and a `target` output path
/// - create an empty file of the correct size on the local file system
/// - determine the optimal piece size (by some heuristic)
/// - iterate over a range from 0 to file's size with step piece size
/// - download a piece of the file at each piece-sized offset
/// - write that piece of the file to the correct offset in the local file
pub fn download_file<P: ThreadsafePath>(
    client: &HttpsClient,
    uri: Uri,
    size: u64,
    target: P,
) -> Box<Future<Item = u64, Error = DlError> + Send> {
    // TODO: consider moving Uri parsing logic inside this function
    // let response_future = client.get(uri).map_err(|err| DlError::Http(err));
    // let file_future = File::create(target).map_err(|err| DlError::Io(err));
    // Box::new(response_future.join(file_future).and_then(write_to_file));
    create_blank_file(target, size);
    Box::new(future::ok(size))
}

/// creates a file of `size` bytes at `path`, containing repeated null bytes
fn create_blank_file<P: ThreadsafePath>(
    path: P,
    size: u64,
) -> impl Future<Item = u64, Error = DlError> {
    File::create(path)
        .map_err(DlError::Io)
        .and_then(move |file| {
            // TODO: improve perf by writing chunks of null bytes instead of one at a time?
            futures::stream::repeat([0u8])
                .take(size)
                .fold(file, write_chunk)
                .and_then(get_file_size)
        })
}

/// writes the contents of a buffer into a file, returning a handle to the file
fn write_chunk<F, B>(file: F, buf: B) -> impl Future<Item = F, Error = DlError>
where
    F: AsyncWrite,
    B: AsRef<[u8]>,
{
    io::write_all(file, buf)
        .map(|(f, _)| f)
        .map_err(DlError::Io)
}

fn get_file_size(file: File) -> impl Future<Item = u64, Error = DlError> {
    file.metadata().map(|(_, md)| md.len()).map_err(DlError::Io)
}

/// TODO: determine optimal piece size for given file size according to these norms:
/// http://wiki.depthstrike.com/index.php/Recommendations#Torrent_Piece_Sizes_when_making_torrents
fn get_piece_size(file_size: usize) -> usize {
    4096
}

#[cfg(test)]
mod download_tests {
    use super::*;
    use crate::checksum;
    use crate::https;
    use tokio::runtime::Runtime;

    const FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
    const FILE_SIZE: u64 = 53143;
    const FILE_MD5_SUM: &'static str = "ac89ac31a669c13ec4ce037f1203022c";
    const ZEROS_MD5_SUM: &'static str = "0f343b0931126a20f133d67c2b018a3b";
    const TARGET_PATH: &'static str = "data/foo.pdf";

    lazy_static! {
        pub static ref CLIENT: HttpsClient = { https::get_client() };
    }

    #[test]
    #[ignore]
    fn downloading_file() {
        // ignore until we implement par
        let uri = FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let result = download_file(&CLIENT, uri, FILE_SIZE, TARGET_PATH)
            .map(|bytes_written| {
                assert_eq!(bytes_written, FILE_SIZE);
                assert!(checksum::md5sum_check(TARGET_PATH, FILE_MD5_SUM).unwrap_or(false));
            })
            .and_then(|_| tokio_fs::remove_file(TARGET_PATH).map_err(|err| DlError::Io(err)));

        rt.block_on(result).unwrap();
    }

    #[test]
    fn creating_blank_file() {
        let mut rt = Runtime::new().unwrap();

        let result = create_blank_file(TARGET_PATH, 1024)
            .map(|bytes_written| {
                assert_eq!(bytes_written, 1024);
                assert!(checksum::md5sum_check(TARGET_PATH, ZEROS_MD5_SUM).unwrap_or(false));
            })
            .and_then(|_| tokio_fs::remove_file(TARGET_PATH).map_err(|err| DlError::Io(err)));

        rt.block_on(result).unwrap();
    }
}
