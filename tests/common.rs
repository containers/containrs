use anyhow::{Context, Result};
use criapi::{
    image_service_client::ImageServiceClient, runtime_service_client::RuntimeServiceClient,
};
use getset::{Getters, MutGetters};
use log::{error, info};
use std::{
    convert::TryFrom,
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
    sync::Once,
    time::Instant,
};
use tempfile::TempDir;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub mod criapi {
    tonic::include_proto!("runtime.v1alpha2");
}

const TIMEOUT: u64 = 2000;
const BINARY_PATH: &str = "target/debug/criserver";

static INIT: Once = Once::new();

#[cfg(test)]
#[ctor::ctor]
fn init() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("Ensuring latest server binary build");
    if let Err(e) = Command::new("cargo").arg("build").status() {
        error!("Unable to build server binary: {}", e);
        exit(1);
    }
}

#[derive(Getters, MutGetters)]
pub struct Sut {
    #[get_mut = "pub"]
    runtime_client: RuntimeServiceClient<Channel>,

    #[allow(dead_code)]
    #[get_mut = "pub"]
    image_client: ImageServiceClient<Channel>,

    #[get = "pub"]
    test_dir: PathBuf,

    pid: u32,
}

impl Sut {
    pub async fn start() -> Result<Sut> {
        INIT.call_once(|| {});

        let tmp_dir = TempDir::new()?;
        info!("Preparing test directory: {}", tmp_dir.path().display());

        let log_path = tmp_dir.path().join("test.log");
        let out_file = File::create(&log_path)?;
        let err_file = out_file.try_clone()?;

        info!("Starting server");
        let sock_path = tmp_dir.path().join("test.sock");
        let child = Command::new(BINARY_PATH)
            .arg("--log-level=debug")
            .arg(format!("--sock-path={}", sock_path.display()))
            .stderr(Stdio::from(err_file))
            .stdout(Stdio::from(out_file))
            .spawn()
            .context("unable to run server")?;

        info!("Waiting for server to be ready");
        Self::check_file_for_output(
            &log_path,
            &sock_path.display().to_string(),
            "Unable to run server",
        )?;
        info!("Server is ready");

        info!("Creating runtime and image service clients");
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                UnixStream::connect(sock_path.clone())
            }))
            .await?;

        Ok(Sut {
            runtime_client: RuntimeServiceClient::new(channel.clone()),
            image_client: ImageServiceClient::new(channel),
            test_dir: tmp_dir.into_path(),
            pid: child.id(),
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        // Stop server
        info!("Killing server pid {}", self.pid);
        Command::new("kill").arg(self.pid.to_string()).status()?;

        // Cleanup temp dir
        info!("Removing test dir {}", self.test_dir().display());
        fs::remove_dir_all(self.test_dir()).context("cleanup test directory")
    }

    fn check_file_for_output(
        file_path: &Path,
        success_pattern: &str,
        failure_pattern: &str,
    ) -> Result<bool> {
        let mut success = false;
        let now = Instant::now();
        let mut reader = BufReader::new(File::open(file_path).context("open output file path")?);

        while now.elapsed().as_secs() < TIMEOUT {
            let mut line = String::new();
            reader.read_line(&mut line).context("read log line")?;
            if !line.is_empty() {
                print!("{}", line);
                if line.contains(success_pattern) {
                    success = true;
                    break;
                }
                if line.contains(failure_pattern) {
                    break;
                }
            }
        }
        return Ok(success);
    }
}
