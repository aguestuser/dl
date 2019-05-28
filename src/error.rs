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
    ParseContentLength,
    ValidFileMetadata,
}

impl fmt::Display for DlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DlError::Http => write!(f, "Failed HTTP request"),
            DlError::ValidFileMetadata => write!(f, "File does not have valid metadata"),
            DlError::ParseContentLength => write!(f, "Failed to parse content length header"),
        }
    }
}

impl Error for DlError {
    fn description(&self) -> &str {
        match *self {
            DlError::Http => "Failed HTTP request",
            DlError::ValidFileMetadata => "File does not have valid metadata",
            DlError::ParseContentLength => "Failed to parse content length header",
        }
    }
}
