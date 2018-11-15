use super::util::handle_request;
use super::HandlerError;

use config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use jobmanager::Job;
use jobmanager::JobManagerRef;
use jobmanager::JobStatus;
use std::borrow::Cow;

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
            let stdout_position = request.stdout_position.unwrap_or(0);
            let stderr_position = request.stderr_position.unwrap_or(0);

            self.job_manager
                .map_job(request.jobid, |job| {
                    Responce::from_job(job, stdout_position, stderr_position)
                }).map_err(|_| HandlerError::new("Job manager error"))?
                .ok_or_else(|| HandlerError::new("Job not found"))
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    jobid: usize,
    stdout_position: Option<usize>,
    stderr_position: Option<usize>,
}

#[derive(Debug, Serialize)]
struct Responce {
    stage: String,
    stdout: String,
    stdout_position: usize,
    stderr: String,
    stderr_position: usize,
    status: Status,
}

#[derive(Debug, Serialize)]
enum Status {
    InProgress,
    Success,
    Failed,
}

impl Responce {
    fn from_job(job: &Job, stdout_position: usize, stderr_position: usize) -> Responce {
        let stage = job
            .stage()
            .cloned()
            .unwrap_or_else(|| String::with_capacity(0));
        let stdout = slice_to_string(job.stdout(), stdout_position);
        let stderr = slice_to_string(job.stderr(), stderr_position);
        let status = match job.status() {
            JobStatus::Complete { success: true } => Status::Success,
            JobStatus::Complete { success: false } => Status::Failed,
            _ => Status::InProgress,
        };

        Responce {
            stage,
            stdout: stdout.into_owned(),
            stdout_position: job.stdout().len(),
            stderr: stderr.into_owned(),
            stderr_position: job.stderr().len(),
            status,
        }
    }
}

fn slice_to_string<'a>(buffer: &'a [u8], position: usize) -> Cow<'a, str> {
    let start = position.min(buffer.len());

    String::from_utf8_lossy(&buffer[start..])
}
