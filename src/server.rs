use crate::config::ConfigRef;
use crate::handler::DestinationHandler;
use crate::handler::JobsHandler;
use crate::handler::RestoreHandler;
use crate::handler::SearchHandler;
use crate::handler::StatusHandler;
use crate::http::HttpClientRef;
use crate::jobmanager::JobManagerRef;
use crate::options::Options;
use crate::pathmanager::PathManagerRef;
use iron::Iron;
use mount::Mount;
use staticfile::Static;

pub fn start(
    options: &Options,
    config: ConfigRef,
    job_manager: JobManagerRef,
    path_manager: PathManagerRef,
    http_client: HttpClientRef,
) -> () {
    let mut mount = Mount::new();
    mount.mount(
        "/api/v1/destination",
        DestinationHandler::new(config.clone()),
    );
    mount.mount(
        "/api/v1/restore",
        RestoreHandler::new(config.clone(), job_manager.clone(), http_client.clone()),
    );
    mount.mount(
        "/api/v1/status",
        StatusHandler::new(config.clone(), job_manager.clone()),
    );
    mount.mount(
        "/api/v1/jobs",
        JobsHandler::new(config.clone(), job_manager.clone()),
    );
    mount.mount(
        "/api/v1/search",
        SearchHandler::new(config.clone(), path_manager.clone()),
    );
    mount.mount("/static", Static::new("public/static"));
    mount.mount("/", Static::new("public"));

    let address = options.address();
    let port = options.port();

    println!("Listening on {}:{}...", address, port);

    match Iron::new(mount).http((address, port)) {
        Ok(_) => {}
        Err(err) => error!("Failed to start HTTP server: {}", err),
    }
}
