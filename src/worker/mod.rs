mod command;
mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use self::command::WorkerCommand;
use self::command::WorkerSettings;

use config::ConfigRef;
use config::Destination;
use jobmanager::JobManagerRef;
use std::collections::HashSet;
use std::path::Path;
use std::thread::Builder;

#[derive(Debug)]
pub struct Worker {
    config: ConfigRef,
    job_manager: JobManagerRef,
    destination: Destination,
    database_name: String,
    ignore_errors: bool,
}

impl Worker {
    pub fn new(
        config: ConfigRef,
        job_manager: JobManagerRef,
        destination: &Destination,
        database_name: &str,
        ignore_errors: bool,
    ) -> Worker {
        Worker {
            config,
            job_manager,
            destination: destination.clone(),
            database_name: database_name.into(),
            ignore_errors,
        }
    }

    fn execute_step<F>(&self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce() -> WorkerResult<()>,
    {
        match callback() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.set_complete(jobid, false)?;

                Err(err)
            }
        }
    }

    fn execute_step_soft<F>(&self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce() -> WorkerResult<()>,
    {
        match callback() {
            Ok(()) => Ok(()),
            Err(_) if self.ignore_errors => Ok(()),
            Err(err) => {
                self.set_complete(jobid, false)?;

                Err(err)
            }
        }
    }

    fn set_complete(&self, jobid: usize, complete: bool) -> WorkerResult<()> {
        self.job_manager
            .set_complete(jobid, complete)
            .map_err(WorkerError::set_status_error)
    }

    fn collect_schema_names(&self, tables: &[String]) -> HashSet<String> {
        let mut result = HashSet::new();

        for table in tables {
            if let Some(index) = table.find('.') {
                result.insert(table[..index].into());
            }
        }

        result
    }

    fn split_table_names(&self, tables: &[String]) -> Vec<(String, String)> {
        let mut result = Vec::new();

        for table in tables {
            if let Some(index) = table.find('.') {
                result.push((table[..index].into(), table[index + 1..].into()));
            }
        }

        result
    }

    fn do_async<F>(self, jobid: usize, callback: F) -> WorkerResult<()>
    where
        F: FnOnce(Worker) -> WorkerResult<()> + Send,
        F: Send + 'static,
    {
        let _ = Builder::new()
            .name(format!("worker #{}", jobid))
            .spawn(move || callback(self))
            .map_err(WorkerError::spawn_thread_error)?;

        Ok(())
    }

    pub fn restore_full(
        self,
        jobid: usize,
        backup_path: &Path,
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();

        self.do_async(jobid, move |worker| {
            let command = WorkerCommand::new(jobid, &worker);

            if drop_database {
                worker.execute_step(jobid, || command.drop_database())?;
            }

            if create_database {
                worker.execute_step(jobid, || command.create_database())?;
            }

            worker.execute_step_soft(jobid, || command.restore_backup(&backup_path))?;
            worker.set_complete(jobid, true)
        })
    }

    pub fn restore_schema(
        self,
        jobid: usize,
        backup_path: &Path,
        schema: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();
        let schema = schema.to_owned();

        self.do_async(jobid, move |worker| {
            let command = WorkerCommand::new(jobid, &worker);

            if drop_database {
                worker.execute_step(jobid, || command.drop_database())?;
            }

            if create_database {
                worker.execute_step(jobid, || command.create_database())?;
            }

            for name in &schema {
                worker.execute_step(jobid, || command.create_schema(name))?;
                worker.execute_step_soft(jobid, || command.restore_schema(name, &backup_path))?;
            }

            worker.set_complete(jobid, true)
        })?;

        Ok(())
    }

    pub fn restore_tables(
        self,
        jobid: usize,
        backup_path: &Path,
        tables: &[String],
        drop_database: bool,
        create_database: bool,
    ) -> WorkerResult<()> {
        let backup_path = backup_path.to_path_buf();
        let tables = tables.to_owned();

        self.do_async(jobid, move |worker| {
            let mut command = WorkerCommand::new(jobid, &worker);

            if drop_database {
                worker.execute_step(jobid, || command.drop_database())?;
            }

            if create_database {
                worker.execute_step(jobid, || command.create_database())?;

                for name in &worker.collect_schema_names(&tables) {
                    worker.execute_step(jobid, || command.create_schema(name))?;
                    worker.execute_step_soft(jobid, || {
                        command.restore_schema_only(name, &backup_path)
                    })?;
                }
            }

            for (schema, table) in &worker.split_table_names(&tables) {
                worker.execute_step(jobid, || command.truncate_table(schema, table))?;
                worker.execute_step_soft(jobid, || {
                    command.restore_table(schema, table, &backup_path)
                })?;
            }

            worker.set_complete(jobid, true)
        })?;

        Ok(())
    }
}

impl WorkerSettings for Worker {
    fn createdb_path(&self) -> &str {
        self.config.commands().createdb_path()
    }

    fn dropdb_path(&self) -> &str {
        self.config.commands().dropdb_path()
    }

    fn pgrestore_path(&self) -> &str {
        self.config.commands().pgrestore_path()
    }

    fn psql_path(&self) -> &str {
        self.config.commands().psql_path()
    }

    fn restore_jobs(&self) -> usize {
        self.config.restore_jobs()
    }

    fn job_manager(&self) -> &JobManagerRef {
        &self.job_manager
    }

    fn host(&self) -> &str {
        self.destination.host()
    }

    fn port(&self) -> u16 {
        self.destination.port()
    }

    fn role(&self) -> &str {
        self.destination.role()
    }

    fn password(&self) -> &str {
        self.destination.password()
    }

    fn database_name(&self) -> &str {
        &self.database_name
    }

    fn ignore_errors(&self) -> bool {
        self.ignore_errors
    }
}
