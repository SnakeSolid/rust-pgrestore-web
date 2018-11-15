mod error;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

use config::ConfigRef;
use pathmanager::PathManagerRef;
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
}

impl Worker {
    fn new(config: ConfigRef, path_manager: PathManagerRef) -> Worker {
        let directories = config
            .search_config()
            .directories()
            .iter()
            .map(|d| d.into())
            .collect();

        Worker {
            interval: config.search_config().interval(),
            path_manager,
            directories,
        }
    }

    fn start(self) {
        if self.directories.is_empty() {
            return;
        }

        loop {
            info!("Start scanning paths");

            let _ = self.path_manager.clear();

            for directory in &self.directories {
                debug!("Scanning {}", directory.display());

                self.scan_directory(directory, 5);
            }

            debug!("Scan complete");

            thread::sleep(Duration::from_secs(self.interval));
        }
    }

    fn scan_directory(&self, path: &PathBuf, recursion_limit: usize) {
        if recursion_limit == 0 {
            return;
        }

        let reader = match fs::read_dir(path) {
            Ok(reader) => reader,
            Err(err) => {
                warn!("Failed to add path - {}", err);

                return;
            }
        };

        for entry in reader {
            let path = entry.unwrap().path();

            if path.is_dir() {
                self.scan_directory(&path, recursion_limit - 1);
            }

            if path.is_file() {
                match path.extension() {
                    Some(extension) if extension == "backup" => {
                        match self.path_manager.add_path(&path) {
                            Ok(()) => {}
                            Err(err) => {
                                warn!("Failed to add path - {}", err);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn start(config: ConfigRef, path_manager: PathManagerRef) {
    if let Err(err) = Builder::new()
        .name(format!("search worker"))
        .spawn(move || Worker::new(config, path_manager).start())
    {
        warn!("Failed to start path worker - {}", err);
    }
}
