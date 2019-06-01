use crate::error::DlError;
use crate::https::HttpsClient;
use futures::{future, stream, Future, Stream};
use hyper;
use hyper::Response;
use hyper::{Body, Request, Uri};
use std::cmp::min;
use std::fmt::Display;
use std::io::SeekFrom;
use std::path::Path;
use tokio_fs::File;
use tokio_io::io;
use tokio_io::AsyncWrite;

// the trait bound describing a typesafe static string to be interpreted as a path by tokio_io
// (it's a mouthful, so we alias it)
pub trait ThreadsafePath: AsRef<Path> + Send + Sync + Display + 'static {}
impl<T: AsRef<Path> + Send + Sync + Display + 'static> ThreadsafePath for T {}

/// given:a http `client`, the `uri` for a file, the file's `size` (in bytes) and a `target` output path
/// download the entire file in sequence and store it to disk, returning a handle to the file
/// NOTE: this function is provided mainly for benchmarking comparison with its parallel counterpart
pub fn download_file_seq<P: ThreadsafePath>(
    client: &HttpsClient,
    uri: Uri,
    path: &'static P,
) -> Box<Future<Item = (), Error = DlError> + Send> {
    // TODO: consider moving Uri parsing logic inside this function
    let response = client.get(uri).map_err(DlError::Hyper);
    let file = File::create(path).map_err(DlError::Io);
    Box::new(
        response
            .join(file)
            .and_then(|(r, f)| write_to_file(r, f, 4096)),
    )
}

/// given a http `client`, the `uri` for a file, the file's `size` (in bytes) and a `target` output path:
/// - create an empty file of the correct size on the local file system
/// - determine the optimal piece size
/// - iterate over a range from 0 to file's size with step piece size
/// - download a piece of the file at each piece-sized offset
/// - write that piece of the file to the correct offset in the local file
pub fn download_file_par<'a, 'b, P: ThreadsafePath>(
    client: &'a HttpsClient,
    uri: Uri,
    file_size: u64,
    piece_size: u64,
    path: &'static P,
) -> impl Future<Item = bool, Error = DlError> + Send + 'a {
    // TODO: consider moving Uri parsing logic inside this function
    create_blank_file(file_size, path).and_then(move |_file_size| {
        // TODO: return error if file is wrong size
        gen_offsets(file_size, piece_size)
            .map(move |offset| download_piece(client, &uri, file_size, piece_size, offset, path))
            .map_err(|_| DlError::StreamProcessing)
            .collect()
            .and_then(|dl_jobs| future::join_all(dl_jobs))
            .map(|_| true)
    })
}

/// creates a file of `size` bytes at `path`, containing repeated null bytes
fn create_blank_file<P: ThreadsafePath>(
    size: u64,
    path: P,
    // ) -> impl Future<Item = u64, Error = DlError> {
) -> impl Future<Item = u64, Error = DlError> {
    File::create(path)
        .map_err(DlError::Io)
        .and_then(move |file| {
            // TODO: improve perf by...
            // - writing chunks of null bytes instead of one at a time?
            // - reserving space without actually writing to it?
            futures::stream::repeat([0u8])
                .take(size)
                .fold(file, |file, buf| write_chunk(file, buf))
                .and_then(get_file_size)
        })
}

pub fn download_piece<'a, P: ThreadsafePath>(
    client: &HttpsClient,
    uri: &Uri,
    file_size: u64,
    piece_size: u64,
    offset: u64,
    path: P,
) -> Box<Future<Item = u64, Error = DlError> + Send> {
    println!(">>> downloading offset: {}", offset);
    match range_request_of(uri, file_size, piece_size, offset) {
        Err(err) => Box::new(future::err(err)),
        Ok(req) => {
            let file = File::open(path).map_err(|err| DlError::Io(err));
            let response = client
                .request(req)
                .map(move |r| log_and_return(r, offset))
                .map_err(|err| DlError::Hyper(err));
            Box::new(
                response
                    .join(file)
                    .and_then(move |(r, f)| write_to_file(r, f, offset).map(move |_| offset)),
            )
        }
    }
}

fn log_and_return(r: Response<Body>, offset: u64) -> Response<Body> {
    println!("!!!!!!! got response for offset: {} !!!!!!!", offset);
    let (status, headers) = (r.status(), r.headers());
    println!("status: {}", status);

    let undefined = hyper::header::HeaderValue::from_static("UNDEFINED");
    // let content_type = headers.get("Content-Type").unwrap_or(&undefined);
    // println!("content_type: {:?}", content_type);

    // let content_length = headers.get("Content-Length").unwrap_or(&undefined);
    // println!("content_length: {:?}", content_length);

    let content_range = headers.get("Content-Range").unwrap_or(&undefined);
    println!("content_range: {:?}", content_range);
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");

    r
}

fn range_request_of(
    uri: &Uri,
    file_size: u64,
    piece_size: u64,
    offset: u64,
) -> Result<Request<Body>, DlError> {
    Request::get(uri)
        .header(
            "Range",
            format!(
                "bytes={}-{}",
                offset,
                min(offset + piece_size, file_size) - 1
            ),
        )
        .body(Body::empty())
        .map_err(DlError::Http)
}

fn write_to_file(
    response: Response<Body>,
    file: File,
    offset: u64,
) -> impl Future<Item = (), Error = DlError> + Send {
    println!("???????? writing at offset: {} ?????????", offset);
    file.seek(SeekFrom::Start(offset))
        .map_err(DlError::Io)
        .and_then(|(file, _)| {
            response
                .into_body()
                .map_err(DlError::Hyper)
                .fold(file, write_chunk)
        })
        .map(drop)
}

/// writes the contents of a buffer into a file, returning a handle to the file
fn write_chunk<F, B>(file: F, buf: B) -> impl Future<Item = F, Error = DlError>
where
    F: AsyncWrite,
    B: AsRef<[u8]>,
{
    io::write_all(file, buf)
        .map(move |(f, _)| f)
        .map_err(DlError::Io)
}

pub fn get_file_size(file: File) -> impl Future<Item = u64, Error = DlError> + Send {
    file.metadata().map(|(_, md)| md.len()).map_err(DlError::Io)
}

/// TODO: determine optimal piece size for given file size according to these norms:
/// http://wiki.depthstrike.com/index.php/Recommendations#Torrent_Piece_Sizes_when_making_torrents
pub fn get_piece_size(_file_size: u64) -> u64 {
    4096
}

// fn calc_offsets(file_size: u64, piece_size: u64) -> Vec<u64> {
//     let (fs, ps) = (file_size as usize, piece_size as usize);
//     (0..fs).step_by(ps).map(|x| x as u64).collect()
// }

fn gen_offsets(file_size: u64, piece_size: u64) -> impl Stream<Item = u64, Error = ()> {
    stream::iter_ok::<_, ()>((0..file_size).step_by(piece_size as usize))
}

#[cfg(test)]
mod download_tests {
    use super::*;
    use crate::checksum;
    use crate::https;
    use tokio::runtime::Runtime;

    const FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
    const FILE_SIZE: u64 = 53143;
    const PIECE_SIZE: u64 = 4096;
    const FILE_MD5_SUM: &'static str = "ac89ac31a669c13ec4ce037f1203022c";
    const ZEROS_MD5_SUM: &'static str = "0f343b0931126a20f133d67c2b018a3b";
    const TARGET_PATH: &'static str = "data/foo.pdf";

    lazy_static! {
        pub static ref CLIENT: HttpsClient = { https::get_client() };
    }

    #[test]
    fn downloading_file_in_sequence() {
        let uri = FILE_URL.parse::<Uri>().unwrap();
        let mut rt = Runtime::new().unwrap();

        let result = download_file_seq(&CLIENT, uri, &TARGET_PATH)
            .and_then(|_| tokio_fs::metadata(TARGET_PATH).map_err(DlError::Io))
            .map(|md| {
                assert_eq!(md.len(), FILE_SIZE);
                assert!(checksum::md5sum_check(TARGET_PATH, FILE_MD5_SUM).unwrap_or(false));
            })
            .and_then(|_| tokio_fs::remove_file(TARGET_PATH).map_err(|err| DlError::Io(err)));

        rt.block_on(result).unwrap();
    }

    #[test]
    fn downloading_file_in_parallel() {
        let uri = FILE_URL.parse::<Uri>().unwrap();
        let piece_size = get_piece_size(FILE_SIZE);
        let mut rt = Runtime::new().unwrap();

        let result = download_file_par(&CLIENT, uri, FILE_SIZE, piece_size, &TARGET_PATH)
            .and_then(|_| tokio_fs::metadata(TARGET_PATH).map_err(DlError::Io))
            .map(|md| {
                assert_eq!(md.len(), FILE_SIZE);
                assert!(checksum::md5sum_check(TARGET_PATH, FILE_MD5_SUM).unwrap_or(false));
            })
            .and_then(|_| tokio_fs::remove_file(TARGET_PATH).map_err(|err| DlError::Io(err)));

        rt.block_on(result).map_err(|e| eprintln!("{}", e)).unwrap();
    }

    // #[test]
    // fn creating_blank_file() {
    //     let mut rt = Runtime::new().unwrap();

    //     let result = create_blank_file(1024, TARGET_PATH)
    //         .map(|file_size| {
    //             assert_eq!(file_size, 1024);
    //             assert!(checksum::md5sum_check(TARGET_PATH, ZEROS_MD5_SUM).unwrap_or(false));
    //         })
    //         .and_then(|_| tokio_fs::remove_file(TARGET_PATH).map_err(|err| DlError::Io(err)));

    //     rt.block_on(result).unwrap();
    // }

    // #[test]
    // fn calculating_offsets() {
    //     assert_eq!(calc_offsets(10, 3), vec![0, 3, 6, 9]);
    //     assert_eq!(
    //         calc_offsets(FILE_SIZE, PIECE_SIZE),
    //         vec![
    //             0, 4096, 8192, 12288, 16384, 20480, 24576, 28672, 32768, 36864, 40960, 45056, 49152
    //         ]
    //     )
    // }

    #[test]
    fn generating_offsets() {
        assert_eq!(
            gen_offsets(10, 3).collect().wait().unwrap(),
            vec![0, 3, 6, 9]
        );
        assert_eq!(
            gen_offsets(FILE_SIZE, PIECE_SIZE).collect().wait().unwrap(),
            vec![
                0, 4096, 8192, 12288, 16384, 20480, 24576, 28672, 32768, 36864, 40960, 45056, 49152
            ]
        )
    }
}
