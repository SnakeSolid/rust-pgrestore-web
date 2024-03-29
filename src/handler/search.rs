use super::util::handle_request;
use super::HandlerError;
use crate::pathmanager::PathManagerRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;

#[derive(Debug)]
pub struct SearchHandler {
    path_manager: PathManagerRef,
}

impl SearchHandler {
    pub fn new(path_manager: PathManagerRef) -> SearchHandler {
        SearchHandler { path_manager }
    }
}

impl Handler for SearchHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let mut result: Vec<String> = Vec::new();

            self.path_manager
                .query_paths(&request.query, 20, |p| result.push(p.display().to_string()))
                .map_err(|_| HandlerError::new("Failed to query results"))?;

            Ok(result)
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    query: String,
}
