use super::util::handle_request;
use super::HandlerError;
use super::HandlerResult;

use config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use jobmanager::Job;
use jobmanager::JobManagerRef;
use jobmanager::JobStatus;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct StatusHandler {
    config: ConfigRef,
    job_manager: JobManagerRef,
}

impl StatusHandler {
    pub fn new(config: ConfigRef, job_manager: JobManagerRef) -> StatusHandler {
        StatusHandler {
            config,
            job_manager,
        }
    }
}

impl Handler for StatusHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let jobid = request.jobid;
            let (database_name, stage, stdout_path, stderr_path, status) = self
                .job_manager
                .map_job(jobid, job_params)
                .map_err(|_| HandlerError::new("Job manager error"))?
                .ok_or_else(|| HandlerError::new("Job not found"))?;
            let stdout_position = request.stdout_position.unwrap_or(0);
            let stderr_position = request.stderr_position.unwrap_or(0);
            let (stdout, stdout_position) = read_file(&stdout_path, stdout_position)?;
            let (stderr, stderr_position) = read_file(&stderr_path, stderr_position)?;

            Ok(Responce {
                database_name,
                stage,
                stdout,
                stdout_position,
                stderr,
                stderr_position,
                status,
            })
        })
    }
}

fn job_params(job: &Job) -> (String, String, PathBuf, PathBuf, Status) {
    let database_name = job.database_name().into();
    let stage = job
        .stage()
        .cloned()
        .unwrap_or_else(|| String::with_capacity(0));
    let stdout_path = job.stdout_path().into();
    let stderr_path = job.stderr_path().into();
    let status = match job.status() {
        JobStatus::Complete { success: true } => Status::Success,
        JobStatus::Complete { success: false } => Status::Failed,
        _ => Status::InProgress,
    };

    (database_name, stage, stdout_path, stderr_path, status)
}

fn read_file(path: &Path, position: u64) -> HandlerResult<(String, u64)> {
    if !path.exists() {
        info!("Job log {} does not exists", path.display());

        return Ok(("".into(), 0));
    }

    if !path.is_file() {
        warn!("Job log {} is not a file", path.display());

        return Err(HandlerError::new("Job log is not a file"));
    }

    let mut file = File::open(path).unwrap();
    let metadata = file
        .metadata()
        .map_err(|_| HandlerError::new("Failed to read job metadata"))?;
    let file_size = metadata.len();
    let start_position = position.min(file_size);

    file.seek(SeekFrom::Start(start_position))
        .map_err(|_| HandlerError::new("Failed to seek job log"))?;

    let mut result = String::with_capacity(file_size as usize - start_position as usize);
    let n = file
        .read_to_string(&mut result)
        .map_err(|_| HandlerError::new("Failed to read job log"))?;

    Ok((result, start_position + n as u64))
}

#[derive(Debug, Deserialize)]
struct Request {
    jobid: usize,
    stdout_position: Option<u64>,
    stderr_position: Option<u64>,
}

#[derive(Debug, Serialize)]
struct Responce {
    database_name: String,
    stage: String,
    stdout: String,
    stdout_position: u64,
    stderr: String,
    stderr_position: u64,
    status: Status,
}

#[derive(Debug, Serialize)]
enum Status {
    InProgress,
    Success,
    Failed,
}
