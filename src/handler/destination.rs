use super::util::handle_empty;
use crate::config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IromRequest;
use iron::Response as IromResponse;

#[derive(Debug)]
pub struct DestinationHandler {
    config: ConfigRef,
}

impl DestinationHandler {
    pub fn new(config: ConfigRef) -> DestinationHandler {
        DestinationHandler { config }
    }
}

impl Handler for DestinationHandler {
    fn handle(&self, _req: &mut IromRequest) -> IronResult<IromResponse> {
        handle_empty(move || {
            let mut result = Vec::new();

            for (index, destination) in self.config.destinations().iter().enumerate() {
                let name = format!(
                    "{}@{}:{}",
                    destination.role(),
                    destination.host(),
                    destination.port()
                );
                let destination = Destination::new(index, &name);

                result.push(destination);
            }

            Ok(result)
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct Destination {
    index: usize,
    name: String,
}

impl Destination {
    fn new(index: usize, name: &str) -> Destination {
        Destination {
            index,
            name: name.into(),
        }
    }
}
