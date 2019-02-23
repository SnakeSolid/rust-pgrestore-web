use crate::config::ConfigRef;
use crate::config::Cors;
use crate::handler::AbortHandler;
use crate::handler::DestinationHandler;
use crate::handler::JobsHandler;
use crate::handler::RestoreHandler;
use crate::handler::SearchHandler;
use crate::handler::StatusHandler;
use crate::http::HttpClientRef;
use crate::jobmanager::JobManagerRef;
use crate::options::Options;
use crate::pathmanager::PathManagerRef;
use iron::Chain;
use iron::Iron;
use iron_cors::CorsMiddleware;
use mount::Mount;
use staticfile::Static;

#[allow(clippy::needless_pass_by_value)]
pub fn start(
    options: &Options,
    config: ConfigRef,
    job_manager: JobManagerRef,
    path_manager: PathManagerRef,
    http_client: HttpClientRef,
) {
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
        "/api/v1/abort",
        AbortHandler::new(config.clone(), job_manager.clone()),
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

    let chain = make_chain(&config, mount);
    let address = options.address();
    let port = options.port();

    println!("Listening on {}:{}...", address, port);

    match Iron::new(chain).http((address, port)) {
        Ok(_) => {}
        Err(err) => error!("Failed to start HTTP server: {}", err),
    }
}

fn make_chain(config: &ConfigRef, mount: Mount) -> Chain {
    let mut chain = Chain::new(mount);

    match config.http_config().cors() {
        Some(Cors::AllowAny) => {
            chain.link_around(CorsMiddleware::with_allow_any());
        }
        Some(Cors::Whitelist { ref whitelist }) => {
            chain.link_around(CorsMiddleware::with_whitelist(whitelist.clone()));
        }
        None => {}
    }

    chain
}
