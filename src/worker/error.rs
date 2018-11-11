use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Debug)]
pub struct WorkerError {
    message: String,
}

impl WorkerError {
    pub fn new(message: &str) -> WorkerError {
        WorkerError {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Error for WorkerError {}

impl Display for WorkerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
