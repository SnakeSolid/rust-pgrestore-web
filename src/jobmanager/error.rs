use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type JobManagerResult<T> = Result<T, JobManagerError>;

#[derive(Debug)]
pub struct JobManagerError {
    message: String,
}

impl JobManagerError {
    pub fn new(message: &str) -> JobManagerError {
        JobManagerError {
            message: message.into(),
        }
    }
}

impl Error for JobManagerError {}

impl Display for JobManagerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
