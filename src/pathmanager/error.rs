use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type PathManagerResult<T> = Result<T, PathManagerError>;

#[derive(Debug)]
pub struct PathManagerError {
    message: String,
}

impl PathManagerError {
    pub fn new(message: &str) -> PathManagerError {
        PathManagerError {
            message: message.into(),
        }
    }
}

impl Error for PathManagerError {}

impl Display for PathManagerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
