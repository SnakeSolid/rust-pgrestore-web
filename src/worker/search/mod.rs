mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use crate::config::ConfigRef;
use crate::pathmanager::PathManagerRef;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::thread::Builder;
use std::time::Duration;

#[derive(Debug)]
pub struct Worker {
    interval: u64,
    path_manager: PathManagerRef,
    directories: Vec<PathBuf>,
    extensions: HashSet<OsString>,
    recursion_limit: usize,
}

const RECURSION_LIMIT: usize = 5;

impl Worker {
    #[allow(clippy::needless_pass_by_value)]
    fn new(config: ConfigRef, path_manager: PathManagerRef) -> Worker {
        let interval = config.search_config().interval();
        let directories = config
            .search_config()
            .directories()
            .iter()
            .map(|d| d.into())
            .collect();
        let extensions = config
            .search_config()
            .extensions()
            .iter()
            .map(|d| d.into())
            .collect();
        let recursion_limit = config
            .search_config()
            .recursion_limit()
            .unwrap_or(RECURSION_LIMIT);

        Worker {
            interval,
            path_manager,
            directories,
            extensions,
            recursion_limit,
        }
    }

    fn start(self) {
        if self.directories.is_empty() {
            info!("No directories to scan, path scanner stopped");

            return;
        }

        if self.extensions.is_empty() {
            info!("No extensions to scan, path scanner stopped");

            return;
        }

        loop {
            info!("Start scanning paths");

            if let Err(err) = self.path_manager.retain(|path| path.is_file()) {
                warn!("Failed to retain old paths - {}", err);
            }

            for directory in &self.directories {
                debug!("Scanning {}", directory.display());

                if let Err(err) = self.scan_directory(directory, self.recursion_limit) {
                    warn!("Directory scan error - {}", err);
                }
            }

            info!("Scan complete");

            thread::sleep(Duration::from_secs(self.interval));
        }
    }

    fn scan_directory(&self, path: &PathBuf, recursion_limit: usize) -> WorkerResult<()> {
        if recursion_limit == 0 {
            info!(
                "Directory {} skipped - Recursion limit exceed",
                path.display()
            );

            return Ok(());
        }

        for entry in fs::read_dir(path).map_err(WorkerError::io_error)? {
            let path = entry.unwrap().path();

            if path.is_dir() {
                if let Err(err) = self.scan_directory(&path, recursion_limit - 1) {
                    warn!("Directory {} skipped - {}", path.display(), err);
                }
            } else if path.is_file() {
                if let Some(extension) = path.extension() {
                    if self.extensions.contains(extension) {
                        if let Err(err) = self
                            .path_manager
                            .add_path(&path)
                            .map_err(WorkerError::add_path_error)
                        {
                            warn!("Failed to add path - {}", err);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn start(config: ConfigRef, path_manager: PathManagerRef) {
    if let Err(err) = Builder::new()
        .name("search worker".to_string())
        .spawn(move || Worker::new(config, path_manager).start())
    {
        warn!("Failed to start path worker - {}", err);
    }
}
