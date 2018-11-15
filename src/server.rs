use config::ConfigRef;
use handler::DestinationHandler;
use handler::RestoreHandler;
use handler::SearchHandler;
use handler::StatusHandler;
use http::HttpClientRef;
use iron::Iron;
use jobmanager::JobManagerRef;
use mount::Mount;
use options::Options;
use pathmanager::PathManagerRef;
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
