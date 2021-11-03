use std::cmp::min;
use std::io::SeekFrom;
use std::path::PathBuf;

use futures::sync::oneshot::Sender;
use futures::{future, oneshot, stream, Future, Stream};
use hyper;
use hyper::Response;
use hyper::{Body, Request, Uri};
use tokio_fs::{File, OpenOptions};
use tokio_io::io;
use tokio_io::AsyncWrite;

use crate::checksum::HashChecker;
use crate::error::DlError;
use crate::https::HttpsClient;
use crate::metadata::Metadata;
use crate::metadata::MetadataDownloader;

pub struct FileDownloader {
    pub client: HttpsClient,
    pub uri: Uri,
    pub path: PathBuf,
    pub file_size: u64,
    pub etag: Option<String>,
    pub parallelism: usize,
}

impl FileDownloader {
    /// constructs a `FileDownloader` from a `MetadataDownloader`
    pub fn from_metadata(mdd: MetadataDownloader, md: Metadata) -> FileDownloader {
        Self {
            client: mdd.client,
            uri: mdd.uri,
            path: mdd.path,
            file_size: md.file_size,
            etag: md.etag,
            parallelism: mdd.parallelism,
        }
    }

    /// given an http `client`, a file's `uri`, a known `file_size`, a desired `piece_size` (in bytes) and an output `path`:
    /// - create an empty file of the correct size on the local file system
    /// - download pieces of the file in parallel
    /// - write each piece to the correct offset in the blank file (also in parallel)
    pub fn fetch(self) -> impl Future<Item = HashChecker, Error = DlError> + Send {
        // TODO increase fault tolerance by:
        //   - inspecting completed futures for success/error retrying all failed requests/writes until no failures
        //   - persisting state of downloads in hashmap, serializing to disk at interval (to be able to restart on crash)
        let Self {
            client,
            file_size,
            path,
            uri,
            etag,
            parallelism,
            ..
        } = self;

        let piece_size = file_size / parallelism as u64;
        let p = path.clone();
        let u = uri.clone();

        File::create(path.clone())
            .map_err(DlError::Io)
            .and_then(move |_| {
                gen_offsets(file_size, piece_size)
                    .map(move |offset| {
                        let (result_sender, result_receiver) = oneshot::<Result<u64, DlError>>();
                        tokio::spawn(download_piece(
                            &client,
                            &u,
                            file_size,
                            piece_size,
                            offset,
                            p.clone(),
                            result_sender,
                        ));
                        result_receiver
                            .wait()
                            .map_or_else(|_| Err(DlError::StreamProcessing), |x| x)
                    })
                    .map_err(|_| DlError::StreamProcessing)
                    .buffer_unordered(parallelism)
                    .collect()
            })
            .map(move |_| HashChecker { path, etag })
    }
}

/// downloads a `piece_size`(d) chunk of the file, seeks to position `offset` and writes the chunk there
pub fn download_piece(
    client: &HttpsClient,
    uri: &Uri,
    file_size: u64,
    piece_size: u64,
    offset: u64,
    path: PathBuf,
    result_sender: Sender<Result<u64, DlError>>,
) -> Box<dyn Future<Item = (), Error = ()> + Send> {
    match build_range_request(uri, file_size, piece_size, offset) {
        Err(err) => {
            let _ = result_sender.send(Err(err));
            Box::new(future::err(()))
        }
        Ok(req) => {
            let response = client.request(req).map_err(|err| DlError::Hyper(err));
            let file = OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(DlError::Io);

            Box::new(
                response
                    .join(file)
                    .and_then(move |(r, f)| write_to_file(r, f, offset))
                    .then(move |res| match res {
                        Ok(_) => {
                            let _ = result_sender.send(Ok(offset));
                            future::ok(())
                        }
                        Err(err) => {
                            let _ = result_sender.send(Err(err));
                            future::err(())
                        }
                    }),
            )
        }
    }
}

/// parses a `response` into a stream and writes it to `offset` in file
fn write_to_file(
    response: Response<Body>,
    file: File,
    offset: u64,
) -> impl Future<Item = File, Error = DlError> + Send {
    file.seek(SeekFrom::Start(offset))
        .map_err(DlError::Io)
        .and_then(move |(file, _)| {
            response
                .into_body()
                .map_err(DlError::Hyper)
                .fold(file, write_chunk)
        })
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

/// builds a range GET request with appropriate begin and end points
fn build_range_request(
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

/// extracts the size (in bytes) from a file on disk
pub fn get_file_size(file: File) -> impl Future<Item = u64, Error = DlError> + Send {
    file.metadata().map(|(_, md)| md.len()).map_err(DlError::Io)
}

fn gen_offsets(file_size: u64, piece_size: u64) -> impl Stream<Item = u64, Error = ()> {
    stream::iter_ok::<_, ()>((0..file_size).step_by(piece_size as usize))
}

#[cfg(test)]
mod download_tests {
    use std::path::Path;

    use tokio::runtime::Runtime;

    use crate::checksum;
    use crate::https;
    use crate::DEFAULT_PARALLELISM;

    use super::*;

    const FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
    const FILE_SIZE: u64 = 53_143;
    const FILE_MD5_SUM: &'static str = "ac89ac31a669c13ec4ce037f1203022c";

    #[test]
    fn downloading_file_in_parallel() {
        let fd = FileDownloader {
            client: https::get_client(*DEFAULT_PARALLELISM),
            uri: FILE_URL.parse::<Uri>().unwrap(),
            path: PathBuf::from("data/foo_par.pdf"),
            file_size: FILE_SIZE,
            etag: None,
            parallelism: *DEFAULT_PARALLELISM,
        };

        let result = fd
            .fetch()
            .and_then(|_| tokio_fs::metadata(Path::new("data/foo_par.pdf")).map_err(DlError::Io))
            .map(|md| {
                assert_eq!(md.len(), FILE_SIZE);
                assert!(
                    checksum::md5sum_check(&Path::new("data/foo_par.pdf"), FILE_MD5_SUM)
                        .unwrap_or(false)
                );
            });

        Runtime::new().unwrap().block_on(result).unwrap();
        std::fs::remove_file(&Path::new("data/foo_par.pdf")).unwrap();
    }

    #[test]
    fn generating_offsets() {
        assert_eq!(
            gen_offsets(10, 3).collect().wait().unwrap(),
            vec![0, 3, 6, 9]
        );
        assert_eq!(
            gen_offsets(FILE_SIZE, 4096).collect().wait().unwrap(),
            vec![
                0, 4096, 8192, 12288, 16384, 20480, 24576, 28672, 32768, 36864, 40960, 45056, 49152
            ]
        )
    }

    #[test]
    fn buffering_a_stream() {
        let results = gen_offsets(64, 2)
            .map(|n| future::ok(n * 2))
            .buffered(8)
            .collect()
            .wait()
            .unwrap();
        assert_eq!(results, (0..128).step_by(4).collect::<Vec<u64>>());
    }
}
