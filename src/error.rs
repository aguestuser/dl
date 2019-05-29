use std::error::Error;
use std::fmt;

/**************************************************************************
 * TODO:
 * - all this custom error boilerplate is pretty gross!
 * - consider `failure` instead: https://boats.gitlab.io/failure/intro.html
 **************************************************************************/

#[derive(Debug)]
pub enum DlError {
    Http,
    Checksum,
    Io(std::io::Error),
    ParseContentLength,
    ValidFileMetadata,
}

impl fmt::Display for DlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DlError::Http => write!(f, "Failed HTTP request"),
            DlError::Io(ref err) => err.fmt(f),
            DlError::Checksum => write!(f, "Failed checksum (hashing or hex encoding failed)"),
            DlError::ParseContentLength => write!(f, "Failed to parse content length header"),
            DlError::ValidFileMetadata => write!(f, "File does not have valid metadata"),
        }
    }
}

impl Error for DlError {
    fn description(&self) -> &str {
        match *self {
            DlError::Http => "Failed HTTP request",
            DlError::Io(ref err) => err.description(),
            DlError::Checksum => "Failed checksum (hashing or hex encoding failed)",
            DlError::ParseContentLength => "Failed to parse content length header",
            DlError::ValidFileMetadata => "File does not have valid metadata",
        }
    }
}

impl From<std::io::Error> for DlError {
    fn from(cause: std::io::Error) -> DlError {
        DlError::Io(cause)
    }
}
