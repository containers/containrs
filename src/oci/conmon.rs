//! Interface to [conmon][0], the OCI container runtime monitor.
//!
//! [0]: https://github.com/containers/conmon

#![allow(dead_code)]

use anyhow::{Context, Result};
use async_trait::async_trait;
use derive_builder::Builder;
use dyn_clone::{clone_trait_object, DynClone};
use getset::{Getters, Setters};
use log::LevelFilter;
use std::{
    fmt::{self, Debug},
    path::{Path, PathBuf},
    process::Output,
    string::ToString,
    time::Duration,
};
use strum::AsRefStr;
use tokio::process::Command;

#[derive(Builder, Debug, Getters, Setters)]
#[builder(pattern = "owned", setter(into))]
// Conmon is the main structure to be used when interacting with the container monitor.
pub struct Conmon {
    #[getset(get, set)]
    #[builder(private, default = "Box::new(DefaultExecCommand)")]
    /// The executor for conmon
    exec: Box<dyn ExecCommand>,

    #[get]
    /// Path to the conmon binary
    binary: PathBuf,
}

impl Conmon {
    /// Run conmon with the provided args and return the output if the command execution succeeds.
    /// This can still mean that conmon itself failed, which can be verified via the exist status
    /// of the output.
    pub async fn run(&self, args: &[Arg]) -> Result<Output> {
        self.exec().run_output(self.binary(), args).await
    }
}

#[derive(Clone, Default, Debug)]
/// DefaultExecCommand is a wrapper which can be used to execute conmon in a standard way.
struct DefaultExecCommand;

impl ExecCommand for DefaultExecCommand {}

#[async_trait]
trait ExecCommand: Debug + DynClone + Send + Sync {
    /// Run a command and return its `Output`.
    async fn run_output(&self, binary: &Path, args: &[Arg]) -> Result<Output> {
        Command::new(binary)
            .args(args.iter().map(ToString::to_string))
            .output()
            .await
            .context("run conmon")
    }
}

clone_trait_object!(ExecCommand);

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for conmon.
pub enum Arg {
    /// Terminal.
    Terminal,

    /// Stdin.
    Stdin,

    /// Leave stdin open when attached client disconnects.
    LeaveStdinOpen,

    /// Container ID.
    Cid(String),

    /// Container UUID.
    Cuuid(String),

    /// Container name.
    Name(String),

    /// Runtime path.
    Runtime(PathBuf),

    /// Restore a container from a checkpoint.
    Restore(String),

    /// Additional opts to pass to the restore or exec command. Can be specified multiple times.
    RuntimeOpt(String),

    /// Additional arg to pass to the runtime. Can be specified multiple times.
    RuntimeArg(String),

    /// Attach to an exec session.
    ExecAttach,

    /// Do not create a new session keyring for the container.
    NoNewKeyring,

    /// Do not use pivot_root.
    NoPivot,

    /// Replace listen pid if set for oci-runtime pid.
    ReplaceListenPid,

    /// Bundle path.
    Bundle(PathBuf),

    /// Persistent directory for a container that can be used for storing container data.
    PersistDir(PathBuf),

    /// Container PID file.
    ContainerPidfile(PathBuf),

    /// Conmon daemon PID file.
    ConmonPidfile(PathBuf),

    /// Enable systemd cgroup manager.
    SystemdCgroup,

    /// Exec a command in a running container.
    Exec,

    /// Conmon API version to use.
    ApiVersion(String),

    /// Path to the process spec for exec.
    ExecProcessSpec(PathBuf),

    /// Path to the directory where exit files are written.
    ExitDir(PathBuf),

    /// Path to the program to execute when the container terminates its execution.
    ExitCommand(PathBuf),

    /// Delay before invoking the exit command (in seconds).
    ExitDelay(Duration),

    /// Additional arg to pass to the exit command. Can be specified multiple times.
    ExitCommandArg(String),

    /// Log file path.
    LogPath(PathBuf),

    /// Timeout in seconds.
    Timeout(Duration),

    /// Maximum size of log file.
    LogSizeMax(u64),

    /// Location of container attach sockets.
    SocketDirPath(PathBuf),

    /// Log to syslog (use with cgroupfs cgroup manager).
    Syslog,

    /// Print debug logs based on log level.
    LogLevel(LevelFilter),

    /// Additional tag to use for logging.
    LogTag(String),

    /// Do not manually call sync on logs after container shutdown.
    NoSyncLog,

    /// Allowing caller to keep the main conmon process as its child by only forking once.
    Sync,
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::conmon::Arg::*;
        write!(f, "--")?;

        fn write_kv<K, V>(f: &mut fmt::Formatter<'_>, key: K, value: V) -> fmt::Result
        where
            K: AsRef<str>,
            V: fmt::Display,
        {
            write!(f, "{}={}", key.as_ref(), value)
        }

        match self {
            Cid(id) => write_kv(f, self, id),
            Cuuid(uuid) => write_kv(f, self, uuid),
            Name(name) => write_kv(f, self, name),
            Runtime(path) => write_kv(f, self, path.display()),
            Restore(checkpoint) => write_kv(f, self, checkpoint),
            RuntimeOpt(opt) => write_kv(f, self, opt),
            RuntimeArg(opt) => write_kv(f, self, opt),
            Bundle(path) => write_kv(f, self, path.display()),
            PersistDir(path) => write_kv(f, self, path.display()),
            ContainerPidfile(path) => write_kv(f, self, path.display()),
            ConmonPidfile(path) => write_kv(f, self, path.display()),
            ApiVersion(version) => write_kv(f, self, version),
            ExecProcessSpec(path) => write_kv(f, self, path.display()),
            ExitDir(path) => write_kv(f, self, path.display()),
            ExitCommand(path) => write_kv(f, self, path.display()),
            ExitDelay(duration) => write_kv(f, self, duration.as_secs()),
            ExitCommandArg(arg) => write_kv(f, self, arg),
            LogPath(path) => write_kv(f, self, path.display()),
            Timeout(duration) => write_kv(f, self, duration.as_secs()),
            LogSizeMax(size) => write_kv(f, self, size),
            SocketDirPath(path) => write_kv(f, self, path.display()),
            LogLevel(level) => write_kv(f, self, level),
            LogTag(tag) => write_kv(f, self, tag),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::process::ExitStatusExt, process::ExitStatus};

    #[derive(Clone, Debug)]
    struct MockExecCommand(Output);

    #[async_trait]
    impl ExecCommand for MockExecCommand {
        async fn run_output(&self, _binary: &Path, _args: &[Arg]) -> Result<Output> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn conmon_success_run() -> Result<()> {
        let conmon = ConmonBuilder::default()
            .binary(which::which("echo")?)
            .build()?;
        let output = conmon.run(&[Arg::Exec]).await?;
        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert_eq!(String::from_utf8(output.stdout)?, "--exec\n");
        Ok(())
    }

    #[tokio::test]
    async fn conmon_success_run_mocked() -> Result<()> {
        let mut conmon = ConmonBuilder::default().binary("").build()?;
        conmon.set_exec(Box::new(MockExecCommand(Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![1, 2, 3],
            stderr: vec![4, 5, 6],
        })));

        let output = conmon
            .run(&[Arg::Cid("cid".into()), Arg::Cuuid("uuid".into())])
            .await?;

        assert!(output.status.success());
        assert_eq!(output.stdout, vec![1, 2, 3]);
        assert_eq!(output.stderr, vec![4, 5, 6]);
        Ok(())
    }

    #[test]
    fn conmon_success() {
        assert!(ConmonBuilder::default()
            .binary("/some/binary")
            .build()
            .is_ok())
    }

    #[test]
    fn conmon_failure_no_binary() {
        assert!(ConmonBuilder::default().build().is_err())
    }

    #[test]
    fn conmon_success_arg_to_string() {
        assert_eq!(&Arg::Terminal.to_string(), "--terminal");
        assert_eq!(
            &Arg::LogPath("/test/path".into()).to_string(),
            "--log-path=/test/path"
        );
        assert_eq!(
            &Arg::LogLevel(LevelFilter::Info).to_string(),
            "--log-level=INFO"
        );
        assert_eq!(&Arg::LogTag("test".into()).to_string(), "--log-tag=test");
        assert_eq!(&Arg::NoSyncLog.to_string(), "--no-sync-log");
        assert_eq!(&Arg::ReplaceListenPid.to_string(), "--replace-listen-pid");
    }
}
