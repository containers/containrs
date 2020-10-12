//! seccomp profile handling

use crate::{
    capability::{Capabilities, Capability},
    oci::spec::runtime::{
        Arch, LinuxSeccomp, LinuxSeccompAction, LinuxSeccompArgBuilder, LinuxSeccompBuilder,
        LinuxSeccompOperator, LinuxSyscall, LinuxSyscallBuilder,
    },
};
use anyhow::{bail, format_err, Context, Result};
use derive_builder::Builder;
use log::debug;
use std::{convert::AsRef, fmt::Display, fs::File, path::PathBuf, string::ToString};

#[derive(Builder, Debug, Default)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
struct Seccomp {
    capability_boundings: Option<Capabilities>,
}

#[derive(Debug)]
enum ProfileType {
    /// The the default internal profile.
    Default,

    /// Seccomp is disabled on purpose.
    Unconfined,

    /// A local path profile.
    Local(PathBuf),
}

impl ProfileType {
    /// Convert a profile name to a profile type.
    fn from(name: &str) -> Result<Self> {
        Ok(match (name, name.strip_prefix("localhost/")) {
            (_, Some(p)) => ProfileType::Local(PathBuf::from(p)),

            (x, _) if x == "runtime/default" || x == "docker/default" => ProfileType::Default,

            (x, _) if x == "unconfined" || x.is_empty() => ProfileType::Unconfined,

            _ => bail!("invalid profile name {}", name),
        })
    }
}

impl Seccomp {
    /// Retrieve the seccomp profile for the provided name.
    ///
    /// Possible values are for `name` are:
    /// - runtime/default: the default profile for the container runtime
    /// - unconfined: unconfined profile, ie, no seccomp sandboxing
    ///   "" is identical with unconfined.
    /// - localhost/<full-path-to-profile>: the profile installed on the node.
    ///   <full-path-to-profile> is the full path of the profile.
    #[allow(dead_code)]
    fn build_linux_seccomp<T>(&self, name: T) -> Result<Option<LinuxSeccomp>>
    where
        T: AsRef<str> + Display,
    {
        Ok(
            match ProfileType::from(name.as_ref()).context("profile name to type")? {
                ProfileType::Default => {
                    debug!("Seccomp profile is {}", name);
                    Some(self.default_profile().context("build default profile")?)
                }

                ProfileType::Unconfined => {
                    debug!("Seccomp profile is unconfined");
                    None
                }

                ProfileType::Local(ref path) => {
                    debug!("Seccomp profile from path {}", path.display());
                    let file = File::open(path)
                        .with_context(|| format!("open file {}", path.display()))?;
                    Some(serde_json::from_reader(file).with_context(|| {
                        format!("deserialize seccomp profile from file {}", path.display())
                    })?)
                }
            },
        )
    }

    /// Build the default profile for the provided capability boundings.
    fn default_profile(&self) -> Result<LinuxSeccomp> {
        let mut syscalls = vec![
            self.default_syscalls()?,
            self.personality_syscall(0x0)?,
            self.personality_syscall(0x0008)?,
            self.personality_syscall(0x20000)?,
            self.personality_syscall(0x20008)?,
            self.personality_syscall(0xffffffff)?,
            #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
            self.arm_syscalls()?,
            #[cfg(target_arch = "x86_64")]
            self.x86_64_syscalls()?,
            #[cfg(target_arch = "x86")]
            self.x86_syscalls()?,
        ];

        if let Some(capabilities) = &self.capability_boundings {
            for capability in capabilities.iter() {
                syscalls.push(
                    LinuxSyscallBuilder::default()
                        .names(
                            self.capability_to_syscalls(*capability)
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>(),
                        )
                        .build()
                        .with_context(|| {
                            format_err!(
                                "build syscalls for {} capability bounding",
                                capability.as_ref()
                            )
                        })?,
                )
            }
        }

        LinuxSeccompBuilder::default()
            .default_action(LinuxSeccompAction::Errno)
            .architectures(DEFAULT_ARCHITECTURES)
            .syscalls(syscalls)
            .build()
            .context("build default profile")
    }

    /// Build allowed syscalls for all architectures.
    fn default_syscalls(&self) -> Result<LinuxSyscall> {
        LinuxSyscallBuilder::default()
            .names(
                DEFAULT_SYSCALLS
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
            .build()
            .context("build default syscalls")
    }

    /// Build an allowed personality syscall for the provided value.
    fn personality_syscall(&self, value: u64) -> Result<LinuxSyscall> {
        LinuxSyscallBuilder::default()
            .names(vec!["personality".into()])
            .args(vec![LinuxSeccompArgBuilder::default()
                .index(0usize)
                .value(value)
                .op(LinuxSeccompOperator::EqualTo)
                .build()
                .context("build personality args")?])
            .build()
            .context("build personality syscall")
    }

    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    /// Build allowed syscalls for the arm architecture.
    fn arm_syscalls(&self) -> Result<LinuxSyscall> {
        LinuxSyscallBuilder::default()
            .names(
                [
                    "arm_fadvise64_64",
                    "arm_sync_file_range",
                    "sync_file_range2",
                    "breakpoint",
                    "cacheflush",
                    "set_tls",
                ]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            )
            .build()
            .context("build arm syscalls")
    }

    #[cfg(target_arch = "x86_64")]
    /// Build allowed syscalls for the x86_64 architecture.
    fn x86_64_syscalls(&self) -> Result<LinuxSyscall> {
        LinuxSyscallBuilder::default()
            .names(
                ["arch_prctl", "modify_ldt"]
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
            .build()
            .context("build x86_64 syscalls")
    }

    #[cfg(target_arch = "x86")]
    /// Build allowed syscalls for the x86 architecture.
    fn x86_syscalls(&self) -> Result<LinuxSyscall> {
        LinuxSyscallBuilder::default()
            .names(vec!["modify_ldt".into()])
            .build()
            .context("build x86 syscall")
    }

    /// Returns a list of syscalls for a provided capability name.
    fn capability_to_syscalls(&self, capability: Capability) -> &'static [&'static str] {
        match capability {
            Capability::DacReadSearch => &["open_by_handle_at"],
            Capability::Syslog => &["syslog"],
            Capability::SysBoot => &["reboot"],
            Capability::SysChroot => &["chroot"],
            Capability::SysModule => &["delete_module", "init_module", "finit_module"],
            Capability::SysPacct => &["acct"],
            Capability::SysPtrace => &["kcmp", "process_vm_readv", "process_vm_writev", "ptrace"],
            Capability::SysRawio => &["iopl", "ioperm"],
            Capability::SysTime => &["settimeofday", "stime", "clock_settime"],
            Capability::SysTtyConfig => &["vhangup"],
            Capability::SysAdmin => &[
                "bpf",
                "clone",
                "fanotify_init",
                "lookup_dcookie",
                "mount",
                "name_to_handle_at",
                "perf_event_open",
                "quotactl",
                "setdomainname",
                "sethostname",
                "setns",
                "syslog",
                "umount",
                "umount2",
                "unshare",
            ],
            _ => &[],
        }
    }
}

const DEFAULT_ARCHITECTURES: &[Arch] = &[
    #[cfg(target_arch = "x86_64")]
    Arch::X86_64,
    #[cfg(target_arch = "x86_64")]
    Arch::X86,
    #[cfg(target_arch = "x86_64")]
    Arch::X32,
    #[cfg(target_arch = "aarch64")]
    Arch::AARCH64,
    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    Arch::ARM,
    #[cfg(any(target_arch = "mips64", target_arch = "mips"))]
    Arch::MIPS,
    #[cfg(target_arch = "mips64")]
    Arch::MIPS64,
    #[cfg(target_arch = "mips64")]
    Arch::MIPS64N32,
    #[cfg(any(target_arch = "powerpc64", target_arch = "powerpc"))]
    Arch::PPC64,
    #[cfg(target_arch = "powerpc")]
    Arch::PPC,
];

const DEFAULT_SYSCALLS: &[&str] = &[
    "accept",
    "accept4",
    "access",
    "adjtimex",
    "alarm",
    "bind",
    "brk",
    "capget",
    "capset",
    "chdir",
    "chmod",
    "chown",
    "chown32",
    "clock_adjtime",
    "clock_adjtime64",
    "clock_getres",
    "clock_getres_time64",
    "clock_gettime",
    "clock_gettime64",
    "clock_nanosleep",
    "clock_nanosleep_time64",
    "close",
    "connect",
    "copy_file_range",
    "creat",
    "dup",
    "dup2",
    "dup3",
    "epoll_create",
    "epoll_create1",
    "epoll_ctl",
    "epoll_ctl_old",
    "epoll_pwait",
    "epoll_wait",
    "epoll_wait_old",
    "eventfd",
    "eventfd2",
    "execve",
    "execveat",
    "exit",
    "exit_group",
    "faccessat",
    "faccessat2",
    "fadvise64",
    "fadvise64_64",
    "fallocate",
    "fanotify_mark",
    "fchdir",
    "fchmod",
    "fchmodat",
    "fchown",
    "fchown32",
    "fchownat",
    "fcntl",
    "fcntl64",
    "fdatasync",
    "fgetxattr",
    "flistxattr",
    "flock",
    "fork",
    "fremovexattr",
    "fsetxattr",
    "fstat",
    "fstat64",
    "fstatat64",
    "fstatfs",
    "fstatfs64",
    "fsync",
    "ftruncate",
    "ftruncate64",
    "futex",
    "futex_time64",
    "futimesat",
    "getcpu",
    "getcwd",
    "getdents",
    "getdents64",
    "getegid",
    "getegid32",
    "geteuid",
    "geteuid32",
    "getgid",
    "getgid32",
    "getgroups",
    "getgroups32",
    "getitimer",
    "getpeername",
    "getpgid",
    "getpgrp",
    "getpid",
    "getppid",
    "getpriority",
    "getrandom",
    "getresgid",
    "getresgid32",
    "getresuid",
    "getresuid32",
    "getrlimit",
    "get_robust_list",
    "getrusage",
    "getsid",
    "getsockname",
    "getsockopt",
    "get_thread_area",
    "gettid",
    "gettimeofday",
    "getuid",
    "getuid32",
    "getxattr",
    "inotify_add_watch",
    "inotify_init",
    "inotify_init1",
    "inotify_rm_watch",
    "io_cancel",
    "ioctl",
    "io_destroy",
    "io_getevents",
    "io_pgetevents",
    "io_pgetevents_time64",
    "ioprio_get",
    "ioprio_set",
    "io_setup",
    "io_submit",
    "io_uring_enter",
    "io_uring_register",
    "io_uring_setup",
    "ipc",
    "kill",
    "lchown",
    "lchown32",
    "lgetxattr",
    "link",
    "linkat",
    "listen",
    "listxattr",
    "llistxattr",
    "_llseek",
    "lremovexattr",
    "lseek",
    "lsetxattr",
    "lstat",
    "lstat64",
    "madvise",
    "membarrier",
    "memfd_create",
    "mincore",
    "mkdir",
    "mkdirat",
    "mknod",
    "mknodat",
    "mlock",
    "mlock2",
    "mlockall",
    "mmap",
    "mmap2",
    "mprotect",
    "mq_getsetattr",
    "mq_notify",
    "mq_open",
    "mq_timedreceive",
    "mq_timedreceive_time64",
    "mq_timedsend",
    "mq_timedsend_time64",
    "mq_unlink",
    "mremap",
    "msgctl",
    "msgget",
    "msgrcv",
    "msgsnd",
    "msync",
    "munlock",
    "munlockall",
    "munmap",
    "nanosleep",
    "newfstatat",
    "_newselect",
    "open",
    "openat",
    "openat2",
    "pause",
    "pipe",
    "pipe2",
    "poll",
    "ppoll",
    "ppoll_time64",
    "prctl",
    "pread64",
    "preadv",
    "preadv2",
    "prlimit64",
    "pselect6",
    "pselect6_time64",
    "pwrite64",
    "pwritev",
    "pwritev2",
    "read",
    "readahead",
    "readlink",
    "readlinkat",
    "readv",
    "recv",
    "recvfrom",
    "recvmmsg",
    "recvmmsg_time64",
    "recvmsg",
    "remap_file_pages",
    "removexattr",
    "rename",
    "renameat",
    "renameat2",
    "restart_syscall",
    "rmdir",
    "rseq",
    "rt_sigaction",
    "rt_sigpending",
    "rt_sigprocmask",
    "rt_sigqueueinfo",
    "rt_sigreturn",
    "rt_sigsuspend",
    "rt_sigtimedwait",
    "rt_sigtimedwait_time64",
    "rt_tgsigqueueinfo",
    "sched_getaffinity",
    "sched_getattr",
    "sched_getparam",
    "sched_get_priority_max",
    "sched_get_priority_min",
    "sched_getscheduler",
    "sched_rr_get_interval",
    "sched_rr_get_interval_time64",
    "sched_setaffinity",
    "sched_setattr",
    "sched_setparam",
    "sched_setscheduler",
    "sched_yield",
    "seccomp",
    "select",
    "semctl",
    "semget",
    "semop",
    "semtimedop",
    "semtimedop_time64",
    "send",
    "sendfile",
    "sendfile64",
    "sendmmsg",
    "sendmsg",
    "sendto",
    "setfsgid",
    "setfsgid32",
    "setfsuid",
    "setfsuid32",
    "setgid",
    "setgid32",
    "setgroups",
    "setgroups32",
    "setitimer",
    "setpgid",
    "setpriority",
    "setregid",
    "setregid32",
    "setresgid",
    "setresgid32",
    "setresuid",
    "setresuid32",
    "setreuid",
    "setreuid32",
    "setrlimit",
    "set_robust_list",
    "setsid",
    "setsockopt",
    "set_thread_area",
    "set_tid_address",
    "setuid",
    "setuid32",
    "setxattr",
    "shmat",
    "shmctl",
    "shmdt",
    "shmget",
    "shutdown",
    "sigaltstack",
    "signalfd",
    "signalfd4",
    "sigprocmask",
    "sigreturn",
    "socket",
    "socketcall",
    "socketpair",
    "splice",
    "stat",
    "stat64",
    "statfs",
    "statfs64",
    "statx",
    "symlink",
    "symlinkat",
    "sync",
    "sync_file_range",
    "syncfs",
    "sysinfo",
    "tee",
    "tgkill",
    "time",
    "timer_create",
    "timer_delete",
    "timer_getoverrun",
    "timer_gettime",
    "timer_gettime64",
    "timer_settime",
    "timer_settime64",
    "timerfd_create",
    "timerfd_gettime",
    "timerfd_gettime64",
    "timerfd_settime",
    "timerfd_settime64",
    "times",
    "tkill",
    "truncate",
    "truncate64",
    "ugetrlimit",
    "umask",
    "uname",
    "unlink",
    "unlinkat",
    "utime",
    "utimensat",
    "utimensat_time64",
    "utimes",
    "vfork",
    "vmsplice",
    "wait4",
    "waitid",
    "waitpid",
    "write",
    "writev",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn from_failure_wrong_name() -> Result<()> {
        assert!(SeccompBuilder::default()
            .build()?
            .build_linux_seccomp("wrong")
            .is_err());
        Ok(())
    }

    #[test]
    fn from_success_unconfined() -> Result<()> {
        assert!(SeccompBuilder::default()
            .build()?
            .build_linux_seccomp("unconfined")?
            .is_none());
        Ok(())
    }

    #[test]
    fn from_success_empty_is_unconfined() -> Result<()> {
        assert!(SeccompBuilder::default()
            .build()?
            .build_linux_seccomp("")?
            .is_none());
        Ok(())
    }

    #[test]
    fn from_success_default() -> Result<()> {
        let profile = SeccompBuilder::default()
            .build()?
            .build_linux_seccomp("runtime/default")?
            .context("no profile")?;
        assert_eq!(profile.default_action(), LinuxSeccompAction::Errno);
        assert_eq!(profile.syscalls().as_ref().context("no syscalls")?.len(), 7);
        Ok(())
    }

    #[test]
    fn from_success_default_capability_boundings() -> Result<()> {
        let profile = SeccompBuilder::default()
            .capability_boundings(Capabilities::all())
            .build()?
            .build_linux_seccomp("runtime/default")?
            .context("no profile")?;
        assert_eq!(
            profile.syscalls().as_ref().context("no syscalls")?.len(),
            48
        );
        Ok(())
    }

    #[test]
    fn from_success_localhost() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        temp_file
            .as_file()
            .write_all(br#"{"defaultAction": "SCMP_ACT_TRACE"}"#)?;

        let profile = SeccompBuilder::default()
            .build()?
            .build_linux_seccomp(format!("localhost/{}", temp_file.path().display()))?
            .context("no profile")?;

        assert_eq!(profile.default_action(), LinuxSeccompAction::Trace);
        Ok(())
    }

    #[test]
    fn from_failure_localhost_wrong_content() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        temp_file.as_file().write_all(b"wrong")?;

        assert!(SeccompBuilder::default()
            .build()?
            .build_linux_seccomp(format!("localhost/{}", temp_file.path().display()))
            .is_err());
        Ok(())
    }

    #[test]
    fn from_failure_localhost_wrong_path() -> Result<()> {
        assert!(SeccompBuilder::default()
            .build()?
            .build_linux_seccomp("localhost/some/wrong/path")
            .is_err());
        Ok(())
    }
}
