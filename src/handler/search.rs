use super::util::handle_request;

use config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;

#[derive(Debug)]
pub struct SearchHandler {
    config: ConfigRef,
}

impl SearchHandler {
    pub fn new(config: ConfigRef) -> SearchHandler {
        SearchHandler { config }
    }
}

impl Handler for SearchHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let query = request.query;
            let mut result: Vec<String> = Vec::new();

            for word in query
                .to_lowercase()
                .split(char::is_whitespace)
                .filter(|t| !t.is_empty())
            {
                result.push(word.into());
            }

            Ok(result)
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    query: String,
}
