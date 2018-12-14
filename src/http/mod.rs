mod error;

pub use self::error::HttpClientError;
pub use self::error::HttpClientResult;

use crate::config::ConfigRef;
use reqwest::Certificate;
use reqwest::Client;
use reqwest::IntoUrl;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct HttpClientRef {
    inner: Arc<Mutex<HttpClient>>,
}

impl HttpClientRef {
    pub fn download<U>(&self, url: U) -> HttpClientResult<PathHandle>
    where
        U: IntoUrl + Display,
    {
        self.inner
            .lock()
            .map_err(HttpClientError::mutex_lock_error)?
            .download(url)
    }
}

#[derive(Debug)]
struct HttpClient {
    config: ConfigRef,
    client: Client,
    download_directory: PathBuf,
    file_seq_no: usize,
}

impl HttpClient {
    fn new(config: ConfigRef) -> HttpClientResult<HttpClient> {
        let mut builder = Client::builder().gzip(true);;

        for cetrificate_path in config.http_config().root_certificates() {
            debug!("Loading certificate {}", cetrificate_path);

            let mut buffer = Vec::new();

            File::open(cetrificate_path)
                .unwrap()
                .read_to_end(&mut buffer)
                .unwrap();

            let certificate = Certificate::from_pem(&buffer).unwrap();

            builder = builder.add_root_certificate(certificate);
        }

        if config.http_config().accept_invalid_hostnames() {
            debug!("Accept invalid host names");

            builder = builder.danger_accept_invalid_hostnames(true);
        }

        if config.http_config().accept_invalid_certs() {
            debug!("Accept invalid certificates");

            builder = builder.danger_accept_invalid_certs(true);
        }

        let download_directory = config.http_config().download_directory().into();

        Ok(HttpClient {
            config,
            client: builder.build().map_err(HttpClientError::reqwest_error)?,
            download_directory,
            file_seq_no: 0,
        })
    }

    pub fn download<U>(&mut self, url: U) -> HttpClientResult<PathHandle>
    where
        U: IntoUrl + Display,
    {
        let file_path = self
            .download_directory
            .join(format!("{}.temp", self.file_seq_no));

        info!("Downloading file {} to {}", url, file_path.display());

        let mut body = self
            .client
            .get(url)
            .send()
            .map_err(HttpClientError::reqwest_error)?;
        let mut writer = File::create(&file_path).map_err(HttpClientError::io_error)?;
        let result = PathHandle::new(file_path);

        body.copy_to(&mut writer)
            .map_err(HttpClientError::reqwest_error)?;

        self.file_seq_no += 1;

        Ok(result)
    }
}

pub fn create(config: ConfigRef) -> HttpClientResult<HttpClientRef> {
    Ok(HttpClientRef {
        inner: Arc::new(Mutex::new(HttpClient::new(config)?)),
    })
}

#[derive(Debug)]
pub struct PathHandle {
    path: PathBuf,
}

impl PathHandle {
    fn new<P>(path: P) -> PathHandle
    where
        P: AsRef<Path>,
    {
        PathHandle {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl AsRef<Path> for PathHandle {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for PathHandle {
    fn drop(&mut self) {
        if self.path.exists() {
            debug!("Removing file {}", self.path.display());

            if let Err(err) = fs::remove_file(&self.path) {
                warn!(
                    "Failed to remove temporary file {} - {}",
                    self.path.display(),
                    err
                );
            }
        }
    }
}
