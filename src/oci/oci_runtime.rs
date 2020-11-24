//! Interface to [oci_runtime][0], a OCI compliant cli for spawning and running containers.
//!
//! [0]: https://github.com/opencontainers/oci_runtime

#![allow(dead_code)]

use anyhow::{Context, Result};
use async_trait::async_trait;
use derive_builder::Builder;
use dyn_clone::{clone_trait_object, DynClone};
use getset::{Getters, Setters};
use std::{
    fmt::{self, Debug},
    path::{Path, PathBuf},
    process::Output,
    string::ToString,
};
use strum::{AsRefStr, Display};
use tokio::process::Command;

#[derive(Builder, Debug, Getters, Setters)]
#[builder(pattern = "owned", setter(into))]
// OCIRuntime is the main structure to be used when interacting with the container runtime.
pub struct OCIRuntime {
    #[getset(get, set)]
    #[builder(private, default = "Box::new(DefaultOCIRuntimeExecCommand)")]
    /// The executor for the OCIRuntime
    exec: Box<dyn ExecCommand>,

    #[get]
    /// Path to the oci_runtime binary
    binary: PathBuf,
}

impl OCIRuntime {
    /// Run OCIRuntime with the provided subcommand and args and return the output if the command execution succeeds.
    /// This can still mean that oci_runtime itself failed, which can be verified via the exist status
    /// of the output.
    pub async fn run(&self, subcommand: &Subcommand, args: &[GlobalArgs]) -> Result<Output> {
        self.exec()
            .run_output(self.binary(), &subcommand.build_cmd()[..], args)
            .await
    }
}

#[derive(Clone, Default, Debug)]
/// DefaultOCIRuntimeExecCommand is a wrapper which can be used to execute OCIRuntime in a standard way.
struct DefaultOCIRuntimeExecCommand;

impl ExecCommand for DefaultOCIRuntimeExecCommand {}

#[async_trait]
trait ExecCommand: Debug + DynClone + Send + Sync {
    /// Run a command and return its `Output`.
    async fn run_output(
        &self,
        binary: &Path,
        cmd: &[String],
        global_args: &[GlobalArgs],
    ) -> Result<Output> {
        Command::new(binary)
            .args(cmd)
            .args(global_args.iter().map(ToString::to_string))
            .output()
            .await
            .context("run OCIRuntime")
    }
}

clone_trait_object!(ExecCommand);

type ContainerId = String;

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Subcommand {
    /// Checkpoint a running container
    Checkpoint((ContainerId, Vec<CheckpointArgs>)),
    /// Create a container
    Create((ContainerId, Vec<CreateArgs>)),
    /// Delete any resources held by the container often used with detached container
    Delete(ContainerId),
    /// Display container events such as OOM notifications, cpu, memory, and IO usage statistics
    Events((ContainerId, Vec<EventsArgs>)),
    /// Execute new process inside the container
    Exec((ContainerId, Vec<ExecArgs>)),
    /// Initialize the namespaces and launch the process (do not call it outside of runc)
    Init,
    /// Kill sends the specified signal (default: SIGTERM) to the container's init process
    Kill((ContainerId, Vec<KillArgs>)),
    /// Lists containers started by runc with the given root
    List(Vec<ListArgs>),
    /// Pause suspends all processes inside the container
    Pause(ContainerId),
    /// Ps displays the processes running inside a container
    Ps((ContainerId, Vec<PsArgs>)),
    /// Restore a container from a previous checkpoint
    Restore((ContainerId, Vec<RestoreArgs>)),
    /// Resumes all processes that have been previously paused
    Resume(ContainerId),
    /// Create and run a container
    Run((ContainerId, Vec<RunArgs>)),
    /// Create a new specification file
    Spec(Vec<SpecArgs>),
    /// Executes the user defined process in a created container
    Start(ContainerId),
    /// Output the state of a container
    State(ContainerId),
    /// Update container resource constraints
    Update((ContainerId, Vec<UpdateArgs>)),
}

impl Subcommand {
    fn build_cmd(&self) -> Vec<String> {
        use crate::oci::oci_runtime::Subcommand::*;
        match self {
            Checkpoint((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Create((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Delete(container_id) => {
                self.build_cmd_vec(Vec::new(), Some(String::from(container_id)))
            }
            Events((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Exec((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Kill((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Pause(container_id) => self.build_cmd_vec(Vec::new(), Some(String::from(container_id))),
            Ps((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Restore((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Resume(container_id) => {
                self.build_cmd_vec(Vec::new(), Some(String::from(container_id)))
            }
            Run((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Start(container_id) => self.build_cmd_vec(Vec::new(), Some(String::from(container_id))),
            State(container_id) => self.build_cmd_vec(Vec::new(), Some(String::from(container_id))),
            Update((container_id, args)) => self.build_cmd_vec(
                args.iter().map(ToString::to_string).collect(),
                Some(String::from(container_id)),
            ),
            Init => self.build_cmd_vec(Vec::new(), None),
            List(args) => self.build_cmd_vec(args.iter().map(ToString::to_string).collect(), None),
            Spec(args) => self.build_cmd_vec(args.iter().map(ToString::to_string).collect(), None),
        }
    }

    /// Build a vec of `[command][args][container_id]`
    fn build_cmd_vec(&self, args: Vec<String>, container_id: Option<String>) -> Vec<String> {
        let mut res = vec![self.to_string()]
            .into_iter()
            .chain(args.into_iter())
            .collect::<Vec<_>>();
        if let Some(id) = container_id {
            res.push(id)
        }
        res
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
/// Available global arguments for oci_runtime.
pub enum RootlessArgs {
    True,
    False,
    Auto,
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
/// Available global arguments for oci_runtime.
pub enum LogFormatArgs {
    Text,
    Json,
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available global arguments for oci_runtime.
pub enum GlobalArgs {
    /// Enable debug output for logging
    Debug,

    /// Set the log file path where internal debug information is written
    Log(PathBuf),

    /// Set the format used by logs ('text' (default), or 'json') (default: "text")
    LogFormat(LogFormatArgs),

    /// Root directory for storage of container state (this should be located in tmpfs) (default: "/run/user/1000/runc")
    Root(PathBuf),

    /// Path to the criu binary used for checkpoint and restore (default: "criu")
    Criu(String),

    /// Enable systemd cgroup support, expects cgroupsPath to be of form "slice:prefix:name" for e.g. "system.slice:runc:434234"
    SystemdCgroup(String),

    /// Ignore cgroup permission errors ('true', 'false', or 'auto') (default: "auto")
    Rootless(RootlessArgs),

    /// Print the version
    Version,
}

fn write_kv<K, V>(f: &mut fmt::Formatter<'_>, key: K, value: V) -> fmt::Result
where
    K: AsRef<str>,
    V: fmt::Display,
{
    write!(f, "{}={}", key.as_ref(), value)
}

impl fmt::Display for GlobalArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::GlobalArgs::*;
        write!(f, "--")?;

        match self {
            Log(path) => write_kv(f, self, path.display()),
            Root(path) => write_kv(f, self, path.display()),
            LogFormat(format) => write_kv(f, self, format),
            Criu(criu) => write_kv(f, self, criu),
            SystemdCgroup(cgroup) => write_kv(f, self, cgroup),
            Rootless(rootless) => write_kv(f, self, rootless),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
/// Available arguments for 'oci_runtime checkpoint'.
pub enum ManageCgroupsModeArgs {
    Soft,
    Full,
    Strict,
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime checkpoint'.
pub enum CheckpointArgs {
    /// Path for saving criu image files
    ImagePath(PathBuf),
    /// Path for saving work files and logs
    WorkPath(PathBuf),
    /// Path for previous criu image files in pre-dump
    ParentPath(PathBuf),
    /// Leave the process running after checkpointing
    LeaveRunning,
    /// Allow open tcp connections
    TcpEstablished,
    /// Allow external unix sockets
    ExtUnixSk,
    /// Allow shell jobs
    ShellJob,
    /// Use userfaultfd to lazily restore memory pages
    LazyPages,
    /// Criu writes \0 to this FD once lazy-pages is ready
    StatusFd(String),
    /// ADDRESS:PORT of the page server
    PageServer(String),
    /// Handle file locks, for safety
    FileLocks,
    /// Dump container's memory information only, leave the container running after this
    PreDump,
    /// cgroups mode: 'soft' (default), 'full' and 'strict'
    ManageCgroupsMode(ManageCgroupsModeArgs),
    /// Create a namespace, but don't restore its properties
    EmptyNs(String),
    /// Enable auto deduplication of memory images
    AutoDedup,
}

impl fmt::Display for CheckpointArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::CheckpointArgs::*;
        write!(f, "--")?;

        match self {
            ImagePath(path) => write_kv(f, self, path.display()),
            WorkPath(path) => write_kv(f, self, path.display()),
            ParentPath(path) => write_kv(f, self, path.display()),
            StatusFd(status) => write_kv(f, self, status),
            PageServer(server) => write_kv(f, self, server),
            ManageCgroupsMode(mode) => write_kv(f, self, mode),
            EmptyNs(ns) => write_kv(f, self, ns),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime create'.
pub enum CreateArgs {
    /// Path to the root of the bundle directory
    Bundle(PathBuf),
    /// Path to an AF_UNIX socket which will receive a file descriptor referencing the master end of the console's pseudoterminal
    ConsoleSocket(PathBuf),
    /// Specify the file to write the process id to
    PidFile(PathBuf),
    /// Do not use pivot root to jail process inside rootfs.  This should be used whenever the rootfs is on top of a ramdisk
    NoPivot,
    /// Do not create a new session keyring for the container.
    NoNewKeyring,
    /// Pass N additional file descriptors to the container (default: 0)
    PreserveFds(u32),
}

impl fmt::Display for CreateArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::CreateArgs::*;
        write!(f, "--")?;

        match self {
            Bundle(path) => write_kv(f, self, path.display()),
            ConsoleSocket(path) => write_kv(f, self, path.display()),
            PidFile(path) => write_kv(f, self, path.display()),
            PreserveFds(n) => write_kv(f, self, n),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime events'.
pub enum EventsArgs {
    /// Set the stats collection interval (default: 5s)
    Interval(u32),
    /// Display the container's stats then exit
    Stats,
}

impl fmt::Display for EventsArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::EventsArgs::*;
        write!(f, "--")?;

        match self {
            Interval(n) => write_kv(f, self, n),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime exec'.
pub enum ExecArgs {
    /// Path to an AF_UNIX socket which will receive a file descriptor referencing the master end of the console's pseudoterminal
    ConsoleSocket(PathBuf),
    /// Current working directory in the container
    Cwd(PathBuf),
    /// Set environment variables
    Env(String),
    /// Allocate a pseudo-TTY
    Ttl,
    /// UID (format: <uid>[:<gid>])
    User(String),
    /// Additional gids
    AdditionalGids(String),
    /// Path to the process.json
    Process(PathBuf),
    /// Detach from the container's process
    Detach,
    /// Specify the file to write the process id to
    PidFile(PathBuf),
    /// Set the asm process label for the process commonly used with selinux
    ProcessLabel(String),
    /// Set the apparmor profile for the process
    Apparmor(String),
    /// Set the no new privileges value for the process
    NoNewPrivs,
    /// Add a capability to the bounding set for the process
    Cap(String),
    /// Pass N additional file descriptors to the container (stdio + $LISTEN_FDS + N in total) (default: 0)
    PreserveFds(u32),
}

impl fmt::Display for ExecArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::ExecArgs::*;
        write!(f, "--")?;

        match self {
            ConsoleSocket(path) => write_kv(f, self, path.display()),
            Cwd(path) => write_kv(f, self, path.display()),
            Env(env) => write_kv(f, self, env),
            User(usr) => write_kv(f, self, usr),
            AdditionalGids(gids) => write_kv(f, self, gids),
            Process(path) => write_kv(f, self, path.display()),
            PidFile(path) => write_kv(f, self, path.display()),
            ProcessLabel(label) => write_kv(f, self, label),
            Apparmor(app) => write_kv(f, self, app),
            Cap(cap) => write_kv(f, self, cap),
            PreserveFds(n) => write_kv(f, self, n),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime kill'.
pub enum KillArgs {
    All,
}

impl fmt::Display for KillArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "--{}", self.as_ref())
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum FormatArgs {
    Table,
    Json,
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime list'.
pub enum ListArgs {
    /// One of: table or json (default: "table")
    Format(FormatArgs),
    /// Display only container IDs
    Quiet,
}

impl fmt::Display for ListArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::ListArgs::*;
        write!(f, "--")?;

        match self {
            Format(val) => write_kv(f, self, val),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime ps'.
pub enum PsArgs {
    ///one of: table or json (default: "table")
    Format(FormatArgs),
}

impl fmt::Display for PsArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::PsArgs::*;
        write!(f, "--")?;

        match self {
            Format(val) => write_kv(f, self, val),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime restore'.
pub enum RestoreArgs {
    /// Path to an AF_UNIX socket which will receive a file descriptor referencing the master end of the console's pseudoterminal
    ConsoleSocket(PathBuf),
    /// Path for saving criu image files
    ImagePath(PathBuf),
    /// Path for saving work files and logs
    WorkPath(PathBuf),
    /// Allow open tcp connections
    TcpEstablished,
    /// Allow external unix sockets
    ExtUnixSk,
    /// Allow shell jobs
    ShellJob,
    /// Handle file locks, for safety
    FileLocks,
    /// cgroups mode: 'soft' (default), 'full' and 'strict'
    ManageCgroupsMode(ManageCgroupsModeArgs),
    /// Path to the root of the bundle directory
    Bundle(PathBuf),
    /// Detach from the container's process
    Detach,
    /// Specify the file to write the process id to
    PidFile(PathBuf),
    /// Disable the use of the subreaper used to reap reparented processes
    NoSubreaper,
    /// Do not use pivot root to jail process inside rootfs.  This should be used whenever the rootfs is on top of a ramdisk
    NoPivot,
    /// Create a namespace, but don't restore its properties
    EmptyNs(String),
    /// Enable auto deduplication of memory images
    AutoDedup,
    /// Use userfaultfd to lazily restore memory pages
    LazyPages,
}

impl fmt::Display for RestoreArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::RestoreArgs::*;
        write!(f, "--")?;

        match self {
            ConsoleSocket(path) => write_kv(f, self, path.display()),
            ImagePath(path) => write_kv(f, self, path.display()),
            WorkPath(path) => write_kv(f, self, path.display()),
            ManageCgroupsMode(mode) => write_kv(f, self, mode),
            Bundle(path) => write_kv(f, self, path.display()),
            PidFile(path) => write_kv(f, self, path.display()),
            EmptyNs(val) => write_kv(f, self, val),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime run'.
pub enum RunArgs {
    /// Path to the root of the bundle directory, defaults to the current directory
    Bundle(PathBuf),
    /// Path to an AF_UNIX socket which will receive a file descriptor referencing the master end of the console's pseudoterminal
    ConsoleSocket(PathBuf),
    /// Detach from the container's process
    Detach,
    /// Specify the file to write the process id to
    PidFile(PathBuf),
    /// Disable the use of the subreaper used to reap reparented processes
    NoSubreaper,
    /// Do not use pivot root to jail process inside rootfs.  This should be used whenever the rootfs is on top of a ramdisk
    NoPivot,
    /// Do not create a new session keyring for the container.  This will cause the container to inherit the calling processes session key
    NoNewKeyright,
    /// Pass N additional file descriptors to the container (stdio + $LISTEN_FDS + N in total) (default: 0)
    PreserveFds(u32),
}

impl fmt::Display for RunArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::RunArgs::*;
        write!(f, "--")?;

        match self {
            Bundle(path) => write_kv(f, self, path.display()),
            ConsoleSocket(path) => write_kv(f, self, path.display()),
            PidFile(path) => write_kv(f, self, path.display()),
            PreserveFds(val) => write_kv(f, self, val),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime spec'.
pub enum SpecArgs {
    /// Path to the root of the bundle directory
    Bundle(PathBuf),
    /// Generate a configuration for a rootless container
    Rootless,
}

impl fmt::Display for SpecArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::SpecArgs::*;
        write!(f, "--")?;

        match self {
            Bundle(path) => write_kv(f, self, path.display()),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Hash, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
/// Available arguments for 'oci_runtime update'.
pub enum UpdateArgs {
    /// Specifies per cgroup weight, range is from 10 to 1000 (default: 0)
    BlkioWeight(u16),
    /// CPU CFS period to be used for hardcapping (in usecs). 0 to use system default
    CpuPeriod(u64),
    /// CPU CFS hardcap limit (in usecs). Allowed cpu time in a given period
    CpuQuota(u64),
    /// CPU shares (relative weight vs. other containers)
    CpuShare(u64),
    /// CPU realtime period to be used for hardcapping (in usecs). 0 to use system default
    CpuRtPeriod(u64),
    /// CPU realtime hardcap limit (in usecs). Allowed cpu time in a given period
    CpuRtRuntime(u64),
    /// CPU(s) to use
    CpusetCpus(u64),
    /// Memory node(s) to use
    CpusetMems(u64),
    /// Kernel memory limit (in bytes)
    KernelMemory(u64),
    /// Kernel memory limit (in bytes) for tcp buffer
    KernelMemoryTcp(u64),
    /// Memory limit (in bytes)
    Memory(u64),
    /// Memory reservation or soft_limit (in bytes)
    MemoryReservation(u64),
    /// Total memory usage (memory + swap); set '-1' to enable unlimited swap
    MemorySwap(u64),
    /// Maximum number of pids allowed in the container (default: 0)
    PidsLimit(u32),
    /// The string of Intel RDT/CAT L3 cache schema
    L3CacheSchema(String),
    /// The string of Intel RDT/MBA memory bandwidth schema
    MemBwSchema(String),
}

impl fmt::Display for UpdateArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::oci::oci_runtime::UpdateArgs::*;
        write!(f, "--")?;

        match self {
            BlkioWeight(val) => write_kv(f, self, val),
            CpuPeriod(val) => write_kv(f, self, val),
            CpuQuota(val) => write_kv(f, self, val),
            CpuShare(val) => write_kv(f, self, val),
            CpuRtPeriod(val) => write_kv(f, self, val),
            CpuRtRuntime(val) => write_kv(f, self, val),
            CpusetCpus(val) => write_kv(f, self, val),
            CpusetMems(val) => write_kv(f, self, val),
            KernelMemory(val) => write_kv(f, self, val),
            KernelMemoryTcp(val) => write_kv(f, self, val),
            Memory(val) => write_kv(f, self, val),
            MemoryReservation(val) => write_kv(f, self, val),
            MemorySwap(val) => write_kv(f, self, val),
            PidsLimit(val) => write_kv(f, self, val),
            L3CacheSchema(val) => write_kv(f, self, val),
            MemBwSchema(val) => write_kv(f, self, val),
        }
    }
}

#[cfg(test)]
mod tests {
    //TODO
    use super::*;

    #[derive(Clone, Debug)]
    struct MockExecCommand(Output);

    #[async_trait]
    impl ExecCommand for MockExecCommand {
        /// Run a command and return its `Output`.
        async fn run_output(
            &self,
            _binary: &Path,
            _cmd: &[String],
            _global_args: &[GlobalArgs],
        ) -> Result<Output> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn ociruntime_success_create() -> Result<()> {
        let runtime = OCIRuntimeBuilder::default()
            .binary(which::which("echo")?)
            .build()?;
        let sc = Subcommand::Create((String::from("id"), vec![CreateArgs::NoPivot]));
        let output = runtime.run(&sc, &vec![GlobalArgs::Debug]).await?;
        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout)?,
            "create --no-pivot id --debug\n"
        );
        Ok(())
    }

    #[tokio::test]
    async fn ociruntime_success_restore() -> Result<()> {
        let runtime = OCIRuntimeBuilder::default()
            .binary(which::which("echo")?)
            .build()?;
        let sc = Subcommand::Restore((
            String::from("id"),
            vec![RestoreArgs::ImagePath(PathBuf::from("some/path"))],
        ));
        let output = runtime.run(&sc, &vec![GlobalArgs::Debug]).await?;
        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout)?,
            "restore --image-path=some/path id --debug\n"
        );
        Ok(())
    }

    #[tokio::test]
    async fn ociruntime_success_run() -> Result<()> {
        let runtime = OCIRuntimeBuilder::default()
            .binary(which::which("echo")?)
            .build()?;
        let sc = Subcommand::Run((String::from("id"), vec![RunArgs::Detach]));
        let output = runtime.run(&sc, &vec![GlobalArgs::Debug]).await?;
        assert!(output.status.success());
        assert!(String::from_utf8(output.stderr)?.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout)?,
            "run --detach id --debug\n"
        );
        Ok(())
    }

    #[test]
    fn ociruntime_failure_no_binary() {
        assert!(OCIRuntimeBuilder::default().build().is_err())
    }

    #[test]
    fn oci_runtime_success_arg_to_string() {
        assert_eq!(&GlobalArgs::Debug.to_string(), "--debug");
        assert_eq!(
            &GlobalArgs::Rootless(RootlessArgs::Auto).to_string(),
            "--rootless=auto"
        );
        assert_eq!(&CheckpointArgs::LeaveRunning.to_string(), "--leave-running");
        assert_eq!(
            &CheckpointArgs::ImagePath("/test/path".into()).to_string(),
            "--image-path=/test/path"
        );
        assert_eq!(
            &CheckpointArgs::TcpEstablished.to_string(),
            "--tcp-established"
        );
        assert_eq!(
            &CreateArgs::Bundle("test".into()).to_string(),
            "--bundle=test"
        );
        assert_eq!(&SpecArgs::Rootless.to_string(), "--rootless");
        assert_eq!(&RestoreArgs::LazyPages.to_string(), "--lazy-pages");
        assert_eq!(&FormatArgs::Json.to_string(), "json");
        assert_eq!(
            &CheckpointArgs::ManageCgroupsMode(ManageCgroupsModeArgs::Full).to_string(),
            "--manage-cgroups-mode=full"
        );
        assert_eq!(
            &ListArgs::Format(FormatArgs::Table).to_string(),
            "--format=table"
        );
    }
}
