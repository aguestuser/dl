use hyper::client::Client;
use hyper::client::HttpConnector;
use hyper::Body;
use hyper_tls::HttpsConnector;

pub type HttpsClient = Client<HttpsConnector<HttpConnector>, Body>;
pub const DEFAULT_POOL_SIZE: usize = 4;

pub fn get_client() -> HttpsClient {
    get_client_of(DEFAULT_POOL_SIZE)
}

pub fn get_client_of(cnxns: usize) -> HttpsClient {
    let mut https = HttpsConnector::new(cnxns).expect("TLS initialization failed");
    https.https_only(true);
    Client::builder().build::<_, hyper::Body>(https)
}

#[cfg(test)]
mod https_tests {
    use super::*;

    #[test]
    fn getting_client() {
        let c = get_client();
        assert_eq!(format!("{:?}", c), "Client");
    }

    #[test]
    fn getting_client_with_n_connections() {
        let c = get_client_of(2);
        assert_eq!(format!("{:?}", c), "Client")
    }
}
