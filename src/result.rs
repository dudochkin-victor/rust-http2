use std::io;
use std::error::Error;
use std::fmt;

use solicit::http::HttpError;


#[derive(Debug)]
pub enum GrpcError {
    Io(io::Error),
    Http(HttpError),
    Other(&'static str),
}

impl Error for GrpcError {
    fn description(&self) -> &str {
        match self {
            &GrpcError::Io(ref err) => err.description(),
            &GrpcError::Http(ref err) => err.description(),
            &GrpcError::Other(ref message) => message,
        }
    }
}

impl fmt::Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &GrpcError::Io(ref err) => write!(f, "io error: {}", err.description()),
            &GrpcError::Http(ref err) => write!(f, "http error: {}", err.description()),
            &GrpcError::Other(ref message) => write!(f, "other error: {}", message),
        }
    }
}

pub type GrpcResult<T> = Result<T, GrpcError>;


impl From<io::Error> for GrpcError {
    fn from(err: io::Error) -> Self {
        GrpcError::Io(err)
    }
}

impl From<HttpError> for GrpcError {
    fn from(err: HttpError) -> Self {
        GrpcError::Http(err)
    }
}

impl From<GrpcError> for io::Error {
    fn from(err: GrpcError) -> io::Error {
        match err {
            GrpcError::Io(e) => e,
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}