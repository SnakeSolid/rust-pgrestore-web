use super::util::handle_request;
use super::HandlerError;
use super::HandlerResult;

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
                .ok_or_else(|| HandlerError::message("Invalid destination id"))?;

            if request.drop_database {
                info!("Dropping database {}", request.database_name);

                let mut command = Command::new(self.config.dropdb_path());
                let mut command = command
                    .env("PGPASSWORD", destination.password())
                    .arg("--host")
                    .arg(destination.host())
                    .arg("--port")
                    .arg(format!("{}", destination.port()))
                    .arg("--username")
                    .arg(destination.role())
                    .arg(&request.database_name);

                execute_command(&mut command)?;
            }

            info!("Creating database {}", request.database_name);

            let mut command = Command::new(self.config.createdb_path());
            let mut command = command
                .env("PGPASSWORD", destination.password())
                .arg("--host")
                .arg(destination.host())
                .arg("--port")
                .arg(format!("{}", destination.port()))
                .arg("--username")
                .arg(destination.role())
                .arg(&request.database_name);

            execute_command(&mut command)?;

            info!(
                "Restoring backup {} to database {}",
                request.backup_path, request.database_name
            );

            let mut command = Command::new(self.config.pgrestore_path());
            let mut command = command
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
                .arg(request.backup_path);

            execute_command(&mut command)?;

            Ok(Responce::with_success(0))
        })
    }
}

fn execute_command(command: &mut Command) -> HandlerResult<()> {
    let status = command
        .spawn()
        .map_err(|err| HandlerError::message(&format!("Failed to drop database - {}", err)))?
        .wait()
        .map_err(|err| HandlerError::message(&format!("Failed to drop database - {}", err)))?;

    if status.success() {
        Ok(())
    } else {
        Err(HandlerError::message(
            "Command returns non success exit code",
        ))
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
