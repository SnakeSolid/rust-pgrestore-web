use super::util::handle_empty;
use crate::config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IromRequest;
use iron::Response as IromResponse;

#[derive(Debug)]
pub struct SettingsHandler {
    config: ConfigRef,
}

impl SettingsHandler {
    pub fn new(config: ConfigRef) -> SettingsHandler {
        SettingsHandler { config }
    }
}

impl Handler for SettingsHandler {
    fn handle(&self, _req: &mut IromRequest) -> IronResult<IromResponse> {
        handle_empty(move || {
            let indexes_available = self.config.indexes_path().is_some();
            let mut destinations = Vec::new();

            for (index, destination) in self.config.destinations().iter().enumerate() {
                let name = format!(
                    "{}@{}:{}",
                    destination.role(),
                    destination.host(),
                    destination.port()
                );
                let destination = Destination::new(index, &name);

                destinations.push(destination);
            }

            Ok(Response {
                indexes_available,
                destinations,
            })
        })
    }
}

#[derive(Debug, Serialize)]
struct Response {
    indexes_available: bool,
    destinations: Vec<Destination>,
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
