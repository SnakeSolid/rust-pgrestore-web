use super::util::handle_request;

use config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use std::process::Command;

#[derive(Debug)]
pub struct RestoreHandler {
    config: ConfigRef,
}

impl RestoreHandler {
    pub fn new(config: ConfigRef) -> RestoreHandler {
        RestoreHandler { config }
    }
}

impl Handler for RestoreHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let destination = &self
                .config
                .destinations()
                .get(request.destination)
                .ok_or("Invalid destination id")
                .unwrap();

            if request.drop_database {
                let mut child = Command::new(self.config.dropdb_path())
                    .env("PGPASSWORD", destination.password())
                    .arg("--host")
                    .arg(destination.host())
                    .arg("--port")
                    .arg(format!("{}", destination.port()))
                    .arg("--username")
                    .arg(destination.role())
                    .arg(&request.database_name)
                    .spawn()
                    .unwrap();

                let status = child.wait().unwrap();

                if !status.success() {}
            }

            let command = Command::new(self.config.createdb_path())
                .env("PGPASSWORD", destination.password())
                .arg("--host")
                .arg(destination.host())
                .arg("--port")
                .arg(format!("{}", destination.port()))
                .arg("--username")
                .arg(destination.role())
                .arg(&request.database_name)
                .spawn();

            command.unwrap().wait().unwrap();

            let command = Command::new(self.config.pgrestore_path())
                .env("PGPASSWORD", destination.password())
                .arg("--host")
                .arg(destination.host())
                .arg("--port")
                .arg(format!("{}", destination.port()))
                .arg("--username")
                .arg(destination.role())
                .arg("--dbname")
                .arg(&request.database_name)
                .arg("--jobs")
                .arg("8")
                .arg(request.backup_path)
                .spawn();

            command.unwrap().wait().unwrap();

            Ok(Responce::with_success(0))
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    destination: usize,
    backup_path: String,
    database_name: String,
    drop_database: bool,
    restore: String,
}

#[derive(Debug, Serialize)]
struct Responce {
    success: bool,
    job_id: usize,
}

impl Responce {
    fn with_success(job_id: usize) -> Responce {
        Responce {
            success: true,
            job_id,
        }
    }
}
