use crate::error::{Result, SandboxError};
use async_trait::async_trait;
use derive_builder::Builder;
use dyn_clone::clone_trait_object;
use dyn_clone::DynClone;
use getset::{Getters, Setters};
use std::fmt::Debug;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Output;
use strum::AsRefStr;
use tokio::process::Command;

#[derive(Builder, Clone, Debug, Getters, Setters)]
#[builder(
    pattern = "owned",
    setter(into, strip_option),
    build_fn(error = "SandboxError")
)]
pub struct Pinns {
    #[get = "pub"]
    binary: PathBuf,
    #[getset(get, set)]
    #[builder(private, default = "Box::new(DefaultExecCommand{})")]
    exec: Box<dyn ExecCommand>,
}

impl Pinns {
    async fn run(&self, args: &[Arg]) -> Result<Output> {
        self.exec()
            .run_output(self.binary(), args)
            .await
            .map_err(|e| SandboxError::Pinning(e.to_string()))
    }
}

impl Default for Pinns {
    fn default() -> Self {
        Self {
            binary: PathBuf::from("pinns"),
            exec: Box::new(DefaultExecCommand {}),
        }
    }
}

#[async_trait]
trait ExecCommand: Debug + DynClone + Send + Sync {
    async fn run_output(&self, binary: &Path, args: &[Arg]) -> Result<Output> {
        Command::new(binary)
            .args(args.iter().map(ToString::to_string))
            .output()
            .await
            .map_err(|e| SandboxError::Pinning(e.to_string()))
    }
}

clone_trait_object!(ExecCommand);

#[derive(Clone, Default, Debug)]
struct DefaultExecCommand {}

impl ExecCommand for DefaultExecCommand {}

#[derive(AsRefStr, Clone, Debug)]
#[strum(serialize_all = "lowercase")]
enum Arg {
    Cgroup,
    Ipc,
    Net,
    Pid,
    Uts,
    Dir(String),
    FileName(String),
    #[strum(serialize = "log-level")]
    LogLevel(LogLevel),
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_kv<K, V>(f: &mut std::fmt::Formatter<'_>, k: K, v: V) -> std::fmt::Result
        where
            K: AsRef<str>,
            V: AsRef<str>,
        {
            write!(f, "{} {}", k.as_ref(), v.as_ref())
        }

        write!(f, "--")?;
        match self {
            Arg::Dir(dir) => write_kv(f, self, dir),
            Arg::FileName(file) => write_kv(f, self, file),
            Arg::LogLevel(level) => write_kv(f, self, level),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug)]
#[strum(serialize_all = "lowercase")]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}
