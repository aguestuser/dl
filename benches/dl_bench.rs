#[macro_use]
extern crate criterion;
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;

use criterion::{Criterion, ParameterizedBenchmark};
use hyper::Uri;
use tokio::runtime::Runtime;

use dl::{file, https};
use file::FileDownloader;

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

fn small_file_varying_parallelism(c: &mut Criterion) {
    c.bench(
        "download small file",
        ParameterizedBenchmark::new(
            "with varying degrees of parallelism",
            move |b, i| {
                b.iter(move || {
                    let res = FileDownloader {
                        client: https::get_client(*i),
                        uri: SMALL_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: SMALL_FILE_SIZE,
                        etag: None,
                        parallelism: *i,
                    }
                    .fetch();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![1, 6, 12, 24, 48],
        )
        .sample_size(20),
    );
}

fn medium_file_varying_parallelism(c: &mut Criterion) {
    c.bench(
        "download medium file",
        ParameterizedBenchmark::new(
            "with varying levels of parallelism",
            move |b, i| {
                b.iter(move || {
                    let res = FileDownloader {
                        client: https::get_client(*i),
                        uri: MEDIUM_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: MEDIUM_FILE_SIZE,
                        etag: None,
                        parallelism: *i,
                    }
                    .fetch();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![1, 6, 12, 24, 48],
        )
        .sample_size(20),
    );
}

fn large_file_varying_parallelism(c: &mut Criterion) {
    c.bench(
        "download large file",
        ParameterizedBenchmark::new(
            "with varying levels of parallelism",
            move |b, i| {
                b.iter(move || {
                    let res = FileDownloader {
                        client: https::get_client(*i),
                        uri: LARGE_FILE_URL.parse::<Uri>().unwrap(),
                        path: PathBuf::from(PATH),
                        file_size: LARGE_FILE_SIZE,
                        etag: None,
                        parallelism: *i,
                    }
                    .fetch();

                    Runtime::new().unwrap().block_on(res).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![1, 6, 12, 24, 48],
        )
        .sample_size(20),
    );
}

criterion_group!(
    benches,
    small_file_varying_parallelism,
    medium_file_varying_parallelism,
    large_file_varying_parallelism,
);
criterion_main!(benches);
