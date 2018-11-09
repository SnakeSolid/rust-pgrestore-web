use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type HandlerResult<T> = Result<T, HandlerError>;

#[derive(Debug)]
pub struct HandlerError {
    message: String,
}

impl HandlerError {
    pub fn message(message: &str) -> HandlerError {
        HandlerError {
            message: message.into(),
        }
    }
}

impl Error for HandlerError {}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
