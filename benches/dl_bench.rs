#[macro_use]
extern crate criterion;
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use criterion::{Criterion, ParameterizedBenchmark};
use dl::{download, https};
use https::HttpsClient;
use tokio::runtime::Runtime;

static PATH: &'static str = "data/foo.pdf";

static SMALL_FILE_URL: &'static str =
    "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";
static SMALL_FILE_SIZE: u64 = 53_143;

static MEDIUM_FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/dc12e4d0-3c82-45b8-9cb7-6c64a8f50cfb/austin_guest_resume.pdf";
static MEDIUM_FILE_SIZE: u64 = 24_975_901;

static LARGE_FILE_URL: &'static str =
    "https://s3.amazonaws.com/ryft-public-sample-data/ODBC/SampleDatabases_3.0.tar.gz";
static LARGE_FILE_SIZE: u64 = 637_828_873;

lazy_static! {
    pub static ref PAR_CLIENT: HttpsClient = { https::get_client() };
    pub static ref SEQ_CLIENT: HttpsClient = { https::get_client() };
}
// TODO: https://docs.rs/hyper/0.12.29/hyper/rt/trait.Stream.html#method.buffered

fn small_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download small file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let mut rt = Runtime::new().unwrap();
                    let result = download::fetch_seq(&SEQ_CLIENT, &SMALL_FILE_URL, &PATH);
                    rt.block_on(result).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let mut rt = Runtime::new().unwrap();
                let result =
                    download::fetch_par(&PAR_CLIENT, &SMALL_FILE_URL, SMALL_FILE_SIZE, &PATH);
                rt.block_on(result).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(10),
    );
}

fn medium_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download medium-sized file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let mut rt = Runtime::new().unwrap();
                    let result = download::fetch_seq(&SEQ_CLIENT, &MEDIUM_FILE_URL, &PATH);
                    rt.block_on(result).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let mut rt = Runtime::new().unwrap();
                let result =
                    download::fetch_par(&PAR_CLIENT, &MEDIUM_FILE_URL, MEDIUM_FILE_SIZE, &PATH);
                rt.block_on(result).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(5),
    );
}

fn large_file_par_vs_seq(c: &mut Criterion) {
    c.bench(
        "download large file",
        ParameterizedBenchmark::new(
            "in sequence",
            |b, _| {
                b.iter(|| {
                    let mut rt = Runtime::new().unwrap();
                    let result = download::fetch_seq(&SEQ_CLIENT, &LARGE_FILE_URL, &PATH);
                    rt.block_on(result).unwrap();
                    std::fs::remove_file(&PATH).unwrap();
                })
            },
            vec![0],
        )
        .with_function("in parallel", |b, _| {
            b.iter(|| {
                let mut rt = Runtime::new().unwrap();
                let result =
                    download::fetch_par(&PAR_CLIENT, &LARGE_FILE_URL, LARGE_FILE_SIZE, &PATH);
                rt.block_on(result).unwrap();
                std::fs::remove_file(&PATH).unwrap();
            })
        })
        .sample_size(2),
    );
}

criterion_group!(
    benches,
    small_file_par_vs_seq,
    medium_file_par_vs_seq,
    large_file_par_vs_seq,
);
criterion_main!(benches);
