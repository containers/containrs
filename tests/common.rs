use anyhow::{bail, Context, Result};
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
    include!("../src/criapi/runtime.v1alpha2.rs");
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

    #[get_mut = "pub"]
    log_file_reader: BufReader<File>,

    #[get = "pub"]
    cni_config_path: PathBuf,
}

impl Sut {
    pub async fn start() -> Result<Self> {
        Self::start_with_args(vec![]).await
    }

    pub async fn start_with_args(args: Vec<String>) -> Result<Self> {
        INIT.call_once(|| {});

        let test_dir = TempDir::new()?.into_path();
        info!("Preparing test directory: {}", test_dir.display());

        let log_path = test_dir.join("test.log");
        let out_file = File::create(&log_path)?;
        let err_file = out_file.try_clone()?;

        // Prepare CNI directory
        let cni_config_path = test_dir.join("cni");
        Command::new("cp")
            .arg("-r")
            .arg("tests/cni")
            .arg(&cni_config_path)
            .output()
            .with_context(|| format!("copy 'cni' test dir to {}", cni_config_path.display()))?;

        info!("Starting server");
        let sock_path = test_dir.join("test.sock");
        let child = Command::new(BINARY_PATH)
            .arg("--log-level=debug")
            .arg(format!("--sock-path={}", sock_path.display()))
            .arg(format!(
                "--storage-path={}",
                test_dir.join("storage").display()
            ))
            .arg(format!("--cni-config-paths={}", cni_config_path.display()))
            .args(args)
            .stderr(Stdio::from(err_file))
            .stdout(Stdio::from(out_file))
            .spawn()
            .context("unable to run server")?;

        info!("Waiting for server to be ready");
        let mut log_file_reader =
            BufReader::new(File::open(log_path).context("open log file path")?);
        if !Self::check_file_for_output(
            &mut log_file_reader,
            &test_dir.display().to_string(),
            "Unable to run server",
        )? {
            bail!("server did not become ready")
        }
        if !Self::wait_for_file_exists(&sock_path)? {
            bail!("socket path {} does not exist", sock_path.display())
        }
        info!("Server is ready");

        info!("Creating runtime and image service clients");
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                UnixStream::connect(sock_path.clone())
            }))
            .await?;

        Ok(Self {
            runtime_client: RuntimeServiceClient::new(channel.clone()),
            image_client: ImageServiceClient::new(channel),
            test_dir,
            pid: child.id(),
            log_file_reader,
            cni_config_path,
        })
    }

    /// Checks if the log file contains the provided line since the last call to this method.
    pub fn log_file_contains_line(&mut self, content: &str) -> Result<bool> {
        let now = Instant::now();

        while now.elapsed().as_secs() < 3 {
            for line_result in self.log_file_reader_mut().lines() {
                let line = line_result.context("read log line")?;
                info!("Got new log line: {}", line);
                if line.contains(content) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn cleanup(&mut self) -> Result<()> {
        // Stop server
        info!("Killing server pid {}", self.pid);
        Command::new("kill").arg(self.pid.to_string()).status()?;

        // Cleanup temp dir
        info!("Removing test dir {}", self.test_dir().display());
        if self.test_dir().exists() {
            fs::remove_dir_all(self.test_dir()).context("cleanup test directory")?;
        }
        Ok(())
    }

    fn check_file_for_output(
        file_reader: &mut BufReader<File>,
        success_pattern: &str,
        failure_pattern: &str,
    ) -> Result<bool> {
        let mut success = false;
        let now = Instant::now();

        while now.elapsed().as_secs() < TIMEOUT {
            let mut line = String::new();
            file_reader.read_line(&mut line).context("read log line")?;
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

    fn wait_for_file_exists(file_path: &Path) -> Result<bool> {
        let mut success = false;
        let now = Instant::now();

        while now.elapsed().as_secs() < TIMEOUT {
            if file_path.exists() {
                success = true;
                break;
            }
        }

        return Ok(success);
    }
}
