use super::util::handle_request;
use super::HandlerError;
use crate::jobmanager::JobManagerRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;

#[derive(Debug)]
pub struct AbortHandler {
    job_manager: JobManagerRef,
}

impl AbortHandler {
    pub fn new(job_manager: JobManagerRef) -> AbortHandler {
        AbortHandler { job_manager }
    }
}

impl Handler for AbortHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            self.job_manager
                .set_aborted(request.jobid)
                .map_err(|_| HandlerError::new("Failed to abort job"))
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    jobid: usize,
}
