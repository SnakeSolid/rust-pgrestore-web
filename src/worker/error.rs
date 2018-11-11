use jobmanager::JobManagerError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

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

    pub fn io_error(error: IoError) -> Self {
        warn!("IO error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn spawn_command_error(error: IoError) -> Self {
        warn!("Spawn command error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn wait_command_error(error: IoError) -> Self {
        warn!("Wait command error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn spawn_thread_error(error: IoError) -> Self {
        warn!("Spawn thread error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn set_stage_error(error: JobManagerError) -> Self {
        warn!("Job manager set stage error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn extend_stdout_error(error: JobManagerError) -> Self {
        warn!("Job manager extend STDOUT error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn extend_stderr_error(error: JobManagerError) -> Self {
        warn!("Job manager extend STDERR error - {}", error);

        WorkerError::new(&format!("{}", error))
    }

    pub fn set_status_error(error: JobManagerError) -> Self {
        warn!("Job manager set status error - {}", error);

        WorkerError::new(&format!("{}", error))
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
