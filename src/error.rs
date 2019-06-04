use http;
use std::error::Error;
use std::fmt;

/**************************************************************************
 * TODO:
 * - all this custom error boilerplate is pretty gross!
 * - consider `failure` instead: https://boats.gitlab.io/failure/intro.html
 **************************************************************************/

#[derive(Debug)]
pub enum DlError {
    Checksum,
    EtagAbsent,
    Http(http::Error),
    Hyper(hyper::error::Error),
    InvalidUri(http::uri::InvalidUri),
    Io(std::io::Error),
    ParseContentLength,
    RangeMetadataAbsent,
    RequestFailed(u16),
    StreamProcessing,
}

impl fmt::Display for DlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DlError::Checksum => write!(f, "Failed checksum (hashing or hex encoding failed)"),
            DlError::EtagAbsent => write!(f, "File does not have an etag"),
            DlError::Http(ref err) => err.fmt(f),
            DlError::Hyper(ref err) => err.fmt(f),
            DlError::InvalidUri(ref err) => err.fmt(f),
            DlError::Io(ref err) => err.fmt(f),
            DlError::ParseContentLength => write!(f, "Failed to parse content length header"),
            DlError::RangeMetadataAbsent => write!(f, "Server does not support range requests"),
            DlError::RequestFailed(code) => write!(f, "Request failed with status code {}", code),
            DlError::StreamProcessing => write!(f, "Stream processing error"),
        }
    }
}

impl Error for DlError {
    fn description(&self) -> &str {
        match *self {
            DlError::Checksum => "Failed checksum (hashing or hex encoding failed)",
            DlError::EtagAbsent => "File does not have an etag",
            DlError::Http(ref err) => err.description(),
            DlError::Hyper(ref err) => err.description(),
            DlError::InvalidUri(ref err) => err.description(),
            DlError::Io(ref err) => err.description(),
            DlError::ParseContentLength => "Failed to parse content length header",
            DlError::RangeMetadataAbsent => "Server does not support range requests",
            DlError::RequestFailed(_) => "Request failed",
            DlError::StreamProcessing => "Stream processing error",
        }
    }
}

impl From<http::uri::InvalidUri> for DlError {
    fn from(cause: http::uri::InvalidUri) -> DlError {
        DlError::InvalidUri(cause)
    }
}

impl From<http::Error> for DlError {
    fn from(cause: http::Error) -> DlError {
        DlError::Http(cause)
    }
}

impl From<hyper::error::Error> for DlError {
    fn from(cause: hyper::error::Error) -> DlError {
        DlError::Hyper(cause)
    }
}

impl From<std::io::Error> for DlError {
    fn from(cause: std::io::Error) -> DlError {
        DlError::Io(cause)
    }
}
