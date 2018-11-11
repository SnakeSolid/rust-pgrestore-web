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
            self.job_manager
                .map_job(request.jobid, Responce::from_job)
                .map_err(|_| HandlerError::new("Job manager error"))?
                .ok_or_else(|| HandlerError::new("Job not found"))
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    jobid: usize,
}

#[derive(Debug, Serialize)]
struct Responce {
    stage: String,
    stdout: String,
    stderr: String,
    status: Status,
}

#[derive(Debug, Serialize)]
enum Status {
    InProgress,
    Success,
    Failed,
}

impl Responce {
    fn from_job(job: &Job) -> Responce {
        let stage = job
            .stage()
            .cloned()
            .unwrap_or_else(|| String::with_capacity(0));
        let stdout_len = job.stdout().len();
        let stderr_len = job.stderr().len();
        let stdout = if stdout_len < 1024 {
            String::from_utf8_lossy(job.stdout())
        } else {
            String::from_utf8_lossy(&job.stdout()[stdout_len - 1024..stdout_len])
        };
        let stderr = if stderr_len < 1024 {
            String::from_utf8_lossy(job.stderr())
        } else {
            String::from_utf8_lossy(&job.stderr()[stdout_len - 1024..stdout_len])
        };
        let status = match job.status() {
            JobStatus::Complete { success: true } => Status::Success,
            JobStatus::Complete { success: false } => Status::Failed,
            _ => Status::InProgress,
        };

        Responce {
            stage,
            stdout: stdout.into_owned(),
            stderr: stderr.into_owned(),
            status,
        }
    }
}
