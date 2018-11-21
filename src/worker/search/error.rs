use pathmanager::PathManagerError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

pub type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Debug)]
pub enum WorkerError {
    IoError { message: String },
    PathManagerError { message: String },
}

impl WorkerError {
    pub fn io_error(error: IoError) -> Self {
        warn!("IO error - {}", error);

        WorkerError::IoError {
            message: format!("{}", error),
        }
    }

    pub fn add_path_error(error: PathManagerError) -> WorkerError {
        warn!("Path manager error - {}", error);

        WorkerError::PathManagerError {
            message: format!("{}", error),
        }
    }
}

impl Error for WorkerError {}

impl Display for WorkerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            WorkerError::IoError { message } => write!(f, "{}", message),
            WorkerError::PathManagerError { message } => write!(f, "{}", message),
        }
    }
}
