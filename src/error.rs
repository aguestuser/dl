use std::error::Error;
use std::fmt;

/**************************************************************************
 * TODO:
 * - all this custom error boilerplate is pretty gross!
 * - consider `failure` instead: https://boats.gitlab.io/failure/intro.html
 **************************************************************************/

#[derive(Debug)]
pub enum DlError {
    Http(hyper::error::Error),
    RequestFailed(u16),
    Checksum,
    Io(std::io::Error),
    ParseContentLength,
    StreamProcessing,
    ValidFileMetadata,
}

impl fmt::Display for DlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DlError::Http(ref err) => err.fmt(f),
            DlError::Io(ref err) => err.fmt(f),
            DlError::Checksum => write!(f, "Failed checksum (hashing or hex encoding failed)"),
            DlError::ParseContentLength => write!(f, "Failed to parse content length header"),
            DlError::RequestFailed(code) => write!(f, "Request failed with status code {}", code),
            DlError::StreamProcessing => write!(f, "Stream processing error"),
            DlError::ValidFileMetadata => write!(f, "File does not have valid metadata"),
        }
    }
}

impl Error for DlError {
    fn description(&self) -> &str {
        match *self {
            DlError::Http(ref err) => err.description(),
            DlError::Io(ref err) => err.description(),
            DlError::Checksum => "Failed checksum (hashing or hex encoding failed)",
            DlError::ParseContentLength => "Failed to parse content length header",
            DlError::RequestFailed(_) => "Request failed",
            DlError::StreamProcessing => "Stream processing error",
            DlError::ValidFileMetadata => "File does not have valid metadata",
        }
    }
}

impl From<hyper::error::Error> for DlError {
    fn from(cause: hyper::error::Error) -> DlError {
        DlError::Http(cause)
    }
}

impl From<std::io::Error> for DlError {
    fn from(cause: std::io::Error) -> DlError {
        DlError::Io(cause)
    }
}
