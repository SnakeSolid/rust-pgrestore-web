use config::ConfigRef;
use handler::DestinationHandler;
use handler::RestoreHandler;
use handler::StatusHandler;
use iron::Iron;
use jobmanager::JobManagerRef;
use mount::Mount;
use options::Options;
use staticfile::Static;

pub fn start(options: &Options, config: ConfigRef, job_manager: JobManagerRef) -> () {
    let mut mount = Mount::new();
    mount.mount(
        "/api/v1/destination",
        DestinationHandler::new(config.clone()),
    );
    mount.mount(
        "/api/v1/restore",
        RestoreHandler::new(config.clone(), job_manager.clone()),
    );
    mount.mount(
        "/api/v1/status",
        StatusHandler::new(config.clone(), job_manager.clone()),
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
