use crate::error::{Result, SandboxError};
use async_trait::async_trait;
use derive_builder::Builder;
use dyn_clone::clone_trait_object;
use dyn_clone::DynClone;
use getset::{CopyGetters, Getters, Setters};
use std::fmt::Debug;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Output;
use strum::{AsRefStr, Display};
use tokio::process::Command;

#[derive(Builder, Clone, Debug, CopyGetters, Getters, Setters)]
#[builder(
    pattern = "owned",
    setter(into, strip_option),
    build_fn(error = "SandboxError")
)]
pub struct Pinns {
    #[get = "pub"]
    #[builder(default = "Pinns::default_pin_tool()?")]
    binary: PathBuf,
    #[get = "pub"]
    #[builder(default = "Pinns::default_pin_dir()?")]
    pin_dir: PathBuf,
    #[get_copy = "pub"]
    #[builder(default = "LogLevel::Info")]
    log_level: LogLevel,
    #[getset(get, set)]
    #[builder(private, default = "Box::new(DefaultExecCommand{})")]
    exec: Box<dyn ExecCommand>,
}

impl Pinns {
    pub(crate) async fn run(&self, args: &[Arg]) -> Result<Output> {
        self.exec().run_output(self.binary(), args).await
    }

    fn default_pin_tool() -> Result<PathBuf> {
        which::which("pinns").map_err(|e| SandboxError::Pinning(e.to_string()))
    }

    fn default_pin_dir() -> Result<PathBuf> {
        Ok(PathBuf::from("/run/containrs"))
    }
}

impl Default for Pinns {
    fn default() -> Self {
        Self {
            binary: Self::default_pin_tool().unwrap(),
            pin_dir: Self::default_pin_dir().unwrap(),
            log_level: Default::default(),
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
pub(crate) enum Arg {
    Cgroup,
    Ipc,
    Net,
    #[allow(dead_code)]
    Pid,
    Uts,
    Dir(PathBuf),
    FileName(String),
    #[strum(serialize = "log-level")]
    LogLevel(LogLevel),
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_kv<K, V>(f: &mut std::fmt::Formatter<'_>, k: K, v: V) -> std::fmt::Result
        where
            K: AsRef<str>,
            V: Display,
        {
            write!(f, "{}={}", k.as_ref(), v)
        }

        write!(f, "--")?;
        match self {
            Arg::Dir(dir) => write_kv(f, self, dir.display()),
            Arg::FileName(file) => write_kv(f, self, file),
            Arg::LogLevel(level) => write_kv(f, self, level),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Display, Clone, Copy, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use which;

    fn setup_pinns() -> Result<Pinns> {
        let pinns = PinnsBuilder::default()
            .binary(which::which("echo")?)
            .build()
            .context("build pinns")?;

        Ok(pinns)
    }

    #[tokio::test]
    async fn pinns_cmd_flags() -> Result<()> {
        let pinns = setup_pinns()?;
        let test_data = &[
            (Arg::Cgroup, "--cgroup\n"),
            (Arg::Ipc, "--ipc\n"),
            (Arg::Net, "--net\n"),
            (Arg::Pid, "--pid\n"),
            (Arg::Uts, "--uts\n"),
        ];

        for t in test_data {
            let output = pinns.run(&[t.0.clone()]).await.context("run pinns")?;
            assert!(output.status.success());
            assert!(String::from_utf8(output.stderr)?.is_empty());
            assert_eq!(String::from_utf8(output.stdout)?, t.1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn pinns_cmd_options() -> Result<()> {
        let pinns = setup_pinns()?;
        let test_data = &[
            (
                Arg::Dir(PathBuf::from("/tmp/containrs")),
                "--dir=/tmp/containrs\n",
            ),
            (
                Arg::FileName("containrs".to_owned()),
                "--filename=containrs\n",
            ),
        ];

        for t in test_data {
            let output = pinns.run(&[t.0.clone()]).await.context("run pinns")?;
            assert!(output.status.success());
            assert!(String::from_utf8(output.stderr)?.is_empty());
            assert_eq!(String::from_utf8(output.stdout)?, t.1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn pinns_cmd_log_level() -> Result<()> {
        let pinns = setup_pinns()?;
        let test_data = &[
            (Arg::LogLevel(LogLevel::Trace), "--log-level=trace\n"),
            (Arg::LogLevel(LogLevel::Debug), "--log-level=debug\n"),
            (Arg::LogLevel(LogLevel::Info), "--log-level=info\n"),
            (Arg::LogLevel(LogLevel::Warn), "--log-level=warn\n"),
            (Arg::LogLevel(LogLevel::Error), "--log-level=error\n"),
            (Arg::LogLevel(LogLevel::Off), "--log-level=off\n"),
        ];

        for t in test_data {
            let output = pinns.run(&[t.0.clone()]).await.context("run pinns")?;
            assert!(output.status.success());
            assert!(String::from_utf8(output.stderr)?.is_empty());
            assert_eq!(String::from_utf8(output.stdout)?, t.1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn pinns_cmd_multiple_args() -> Result<()> {
        let pinns = setup_pinns()?;
        let args = &[
            Arg::Ipc,
            Arg::Uts,
            Arg::Net,
            Arg::Dir(PathBuf::from("/tmp/containrs")),
            Arg::FileName("containrs".to_owned()),
            Arg::LogLevel(LogLevel::Warn),
        ];

        let output = pinns.run(args).await.context("run pinns")?;
        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout)?,
            "--ipc --uts --net --dir=/tmp/containrs --filename=containrs --log-level=warn\n"
        );

        Ok(())
    }

    #[test]
    fn default_values_set() -> Result<()> {
        let pinns = PinnsBuilder::default()
            .binary(which::which("echo")?)
            .build()?;

        assert_eq!(pinns.pin_dir(), &PathBuf::from("/run/containrs"));
        Ok(())
    }
}
