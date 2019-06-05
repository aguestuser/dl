#[macro_use]
extern crate criterion;
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use criterion::{Criterion, ParameterizedBenchmark};
use dl::{file, https};
use file::FileDownloader;
use file::DEFAULT_NUM_PIECES;
use https::HttpsClient;
use hyper::Uri;
use std::path::PathBuf;
use tokio::runtime::Runtime;

static PATH: &'static str = "data/foo.pdf";

static SMALL_FILE_URL: &'static str =
    "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
static SMALL_FILE_SIZE: u64 = 53_143;

static MEDIUM_FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/dc12e4d0-3c82-45b8-9cb7-6c64a8f50cfb/austin_guest_resume.pdf";
static MEDIUM_FILE_SIZE: u64 = 24_975_901;

static LARGE_FILE_URL: &'static str =
    "https://recurse-uploads-production.s3.amazonaws.com/cb60706d-3a65-42cc-bfb4-effc9e81f1f8/austin_guest_resume.pdf";
static LARGE_FILE_SIZE: u64 = 637_828_873;

static VERY_LARGE_FILE_URL: &'static str =
    "https://gensho.ftp.acc.umu.se/debian-cd/current-live/amd64/iso-hybrid/debian-live-9.9.0-amd64-xfce.iso";
static VERY_LARGE_FILE_SIZE: u64 = 1_951_432_704;

// TODO: https://docs.rs/hyper/0.12.29/hyper/rt/trait.Stream.html#method.buffered

fn small_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download small file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: SMALL_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: 0,
                        etag: None,
                        num_pieces: 0,
                    }
                    .fetch_seq();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let res = FileDownloader {
                    client: https::get_client(),
                    uri: SMALL_FILE_URL.parse::<Uri>().unwrap(),
                    path: PathBuf::from(PATH),
                    file_size: SMALL_FILE_SIZE,
                    etag: None,
                    num_pieces: DEFAULT_NUM_PIECES,
                }
                .fetch();

                Runtime::new().unwrap().block_on(res).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(10),
    );
}

fn medium_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download medium file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: MEDIUM_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: 0,
                        etag: None,
                        num_pieces: 0,
                    }
                    .fetch_seq();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let res = FileDownloader {
                    client: https::get_client(),
                    uri: MEDIUM_FILE_URL.parse::<Uri>().unwrap(),
                    path: PathBuf::from(PATH),
                    file_size: MEDIUM_FILE_SIZE,
                    etag: None,
                    num_pieces: DEFAULT_NUM_PIECES,
                }
                .fetch();

                Runtime::new().unwrap().block_on(res).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(5),
    );
}

fn medium_file_par_varying_piece_sizes(c: &mut Criterion) {
    c.bench(
        "download medium file",
        ParameterizedBenchmark::new(
            "in parallel",
            move |b, i| {
                b.iter(move || {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: MEDIUM_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: MEDIUM_FILE_SIZE,
                        etag: None,
                        num_pieces: *i,
                    }
                    .fetch();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![8, 16, 32, 64, 128, 256],
        )
        .sample_size(10),
    );
}

fn large_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download large file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: LARGE_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: 0,
                        etag: None,
                        num_pieces: 0,
                    }
                    .fetch_seq();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let res = FileDownloader {
                    client: https::get_client(),
                    uri: LARGE_FILE_URL.parse::<Uri>().unwrap(),
                    path: PathBuf::from(PATH),
                    file_size: LARGE_FILE_SIZE,
                    etag: None,
                    num_pieces: 64,
                }
                .fetch();

                Runtime::new().unwrap().block_on(res).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(2),
    );
}

fn large_file_par_varying_piece_sizes(c: &mut Criterion) {
    c.bench(
        "download large file",
        ParameterizedBenchmark::new(
            "in parallel",
            move |b, i| {
                b.iter(move || {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: LARGE_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: LARGE_FILE_SIZE,
                        etag: None,
                        num_pieces: *i,
                    }
                    .fetch();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![8, 16, 32],
        )
        .sample_size(2),
    );
}

fn large_file_par(c: &mut Criterion) {
    c.bench(
        "download large file",
        ParameterizedBenchmark::new(
            "in parallel",
            move |b, i| {
                b.iter(move || {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: LARGE_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: LARGE_FILE_SIZE,
                        etag: None,
                        num_pieces: 64,
                    }
                    .fetch();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
            .sample_size(2),
    );
}

fn very_large_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download very large file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let res = FileDownloader {
                        client: https::get_client(),
                        uri: VERY_LARGE_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: 0,
                        etag: None,
                        num_pieces: 0,
                    }
                    .fetch_seq();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let res = FileDownloader {
                    client: https::get_client(),
                    uri: VERY_LARGE_FILE_URL.parse::<Uri>().unwrap(),
                    path: PathBuf::from(PATH),
                    file_size: VERY_LARGE_FILE_SIZE,
                    etag: None,
                    num_pieces: 16,
                }
                .fetch();

                Runtime::new().unwrap().block_on(res).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(2),
    );
}



criterion_group!(
    benches,
    // medium_file_par_varying_piece_sizes,
    // large_file_par_varying_piece_sizes,
    // small_file_par_vs_seq,
    // medium_file_par_vs_seq,
    // large_file_par_vs_seq,
    // large_file_par,
    very_large_file_par_vs_seq,
);
criterion_main!(benches);
