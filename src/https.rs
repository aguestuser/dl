use hyper::client::Client;
use hyper::client::HttpConnector;
use hyper::Body;
use hyper_tls::HttpsConnector;

pub type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;

/// returns a (hyper) async https client with threadpool of given size
pub fn get_client(pool_size: usize) -> HttpsClient {
    get_client_of(pool_size)
}

pub fn get_client_of(thread_pool_size: usize) -> HttpsClient {
    let mut https = HttpsConnector::new(thread_pool_size).expect("TLS initialization failed");
    https.https_only(true);
    Client::builder().build::<_, hyper::Body>(https)
}

#[cfg(test)]
mod https_tests {
    use super::*;
    use crate::DEFAULT_PARALLELISM;

    #[test]
    fn getting_client() {
        let c = get_client(*DEFAULT_PARALLELISM);
        assert_eq!(format!("{:?}", c), "Client");
    }

    #[test]
    fn getting_client_with_n_connections() {
        let c = get_client_of(2);
        assert_eq!(format!("{:?}", c), "Client")
    }
}
