//! Linux capability handling

use std::{collections::HashSet, ops::Deref};
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug)]
/// A set of capabilities.
pub struct Capabilities(HashSet<Capability>);

impl Capabilities {
    #[allow(dead_code)]
    /// Get all capabilities.
    pub fn all() -> Self {
        Self(Capability::iter().collect())
    }
}

impl Deref for Capabilities {
    type Target = HashSet<Capability>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    AsRefStr, IntoStaticStr, Copy, Clone, Debug, Display, EnumIter, EnumString, Eq, Hash, PartialEq,
)]
#[strum(serialize_all = "shouty_snake_case")]
/// All available capabilities.
pub enum Capability {
    #[strum(serialize = "CHOWN", serialize = "CAP_CHOWN")]
    // In a system with the [_POSIX_CHOWN_RESTRICTED] option defined, this
    // overrides the restriction of changing file ownership and group ownership.
    Chown,

    #[strum(serialize = "DAC_OVERRIDE", serialize = "CAP_DAC_OVERRIDE")]
    // Override all DAC access, including ACL execute access if [_POSIX_ACL] is
    // defined. Excluding DAC access covered by CAP_LINUX_IMMUTABLE.
    DacOverride,

    #[strum(serialize = "DAC_READ_SEARCH", serialize = "CAP_DAC_READ_SEARCH")]
    // Overrides all DAC restrictions regarding read and search on files
    // and directories, including ACL restrictions if [_POSIX_ACL] is
    // defined. Excluding DAC access covered by CAP_LINUX_IMMUTABLE.
    DacReadSearch,

    #[strum(serialize = "FOWNER", serialize = "CAP_FOWNER")]
    // Overrides all restrictions about allowed operations on files, where
    // file owner ID must be equal to the user ID, except where CAP_FSETID
    // is applicable. It doesn't override MAC and DAC restrictions.
    Fowner,

    #[strum(serialize = "FSETID", serialize = "CAP_FSETID")]
    // Overrides the following restrictions that the effective user ID
    // shall match the file owner ID when setting the S_ISUID and S_ISGID
    // bits on that file; that the effective group ID (or one of the
    // supplementary group IDs) shall match the file owner ID when setting
    // the S_ISGID bit on that file; that the S_ISUID and S_ISGID bits are
    // cleared on successful return from chown(2) (not implemented).
    Fsetid,

    #[strum(serialize = "KILL", serialize = "CAP_KILL")]
    // Overrides the restriction that the real or effective user ID of a
    // process sending a signal must match the real or effective user ID
    // of the process receiving the signal.
    Kill,

    #[strum(serialize = "SETGID", serialize = "CAP_SETGID")]
    // Allows setgid(2) manipulation
    // Allows setgroups(2)
    // Allows forged gids on socket credentials passing.
    Setgid,

    #[strum(serialize = "SETUID", serialize = "CAP_SETUID")]
    // Allows set*uid(2) manipulation (including fsuid).
    // Allows forged pids on socket credentials passing.
    Setuid,

    #[strum(serialize = "SETPCAP", serialize = "CAP_SETPCAP")]
    // Without VFS support for capabilities:
    //   Transfer any capability in your permitted set to any pid,
    //   remove any capability in your permitted set from any pid
    // With VFS support for capabilities (neither of above, but)
    //   Add any capability from current's capability bounding set
    //     to the current process' inheritable set
    //   Allow taking bits out of capability bounding set
    //   Allow modification of the securebits for a process
    Setpcap,

    #[strum(serialize = "LINUX_IMMUTABLE", serialize = "CAP_LINUX_IMMUTABLE")]
    // Allow modification of S_IMMUTABLE and S_APPEND file attributes
    LinuxImmutable,

    #[strum(serialize = "NET_BIND_SERVICE", serialize = "CAP_NET_BIND_SERVICE")]
    // Allows binding to TCP/UDP sockets below 1024
    // Allows binding to ATM VCIs below 32
    NetBindService,

    #[strum(serialize = "NET_BROADCAST", serialize = "CAP_NET_BROADCAST")]
    // Allow broadcasting, listen to multicast
    NetBroadcast,

    #[strum(serialize = "NET_ADMIN", serialize = "CAP_NET_ADMIN")]
    // Allow interface configuration
    // Allow administration of IP firewall, masquerading and accounting
    // Allow setting debug option on sockets
    // Allow modification of routing tables
    // Allow setting arbitrary process / process group ownership on
    // sockets
    // Allow binding to any address for transparent proxying (also via NET_RAW)
    // Allow setting TOS (type of service)
    // Allow setting promiscuous mode
    // Allow clearing driver statistics
    // Allow multicasting
    // Allow read/write of device-specific registers
    // Allow activation of ATM control sockets
    NetAdmin,

    #[strum(serialize = "NET_RAW", serialize = "CAP_NET_RAW")]
    // Allow use of RAW sockets
    // Allow use of PACKET sockets
    // Allow binding to any address for transparent proxying (also via
    // NET_ADMIN)
    NetRaw,

    #[strum(serialize = "IPC_LOCK", serialize = "CAP_IPC_LOCK")]
    // Allow locking of shared memory segments
    // Allow mlock and mlockall (which doesn't really have anything to do
    // with IPC)
    IpcLock,

    #[strum(serialize = "IPC_OWNER", serialize = "CAP_IPC_OWNER")]
    // Override IPC ownership checks
    IpcOwner,

    #[strum(serialize = "SYS_MODULE", serialize = "CAP_SYS_MODULE")]
    // Insert and remove kernel modules - modify kernel without limit
    SysModule,

    #[strum(serialize = "SYS_RAWIO", serialize = "CAP_SYS_RAWIO")]
    // Allow ioperm/iopl access
    // Allow sending USB messages to any device via /proc/bus/usb
    SysRawio,

    #[strum(serialize = "SYS_CHROOT", serialize = "CAP_SYS_CHROOT")]
    // Allow use of chroot()
    SysChroot,

    #[strum(serialize = "SYS_PTRACE", serialize = "CAP_SYS_PTRACE")]
    // Allow ptrace() of any process
    SysPtrace,

    #[strum(serialize = "SYS_PACCT", serialize = "CAP_SYS_PACCT")]
    // Allow configuration of process accounting
    SysPacct,

    #[strum(serialize = "SYS_ADMIN", serialize = "CAP_SYS_ADMIN")]
    // Allow configuration of the secure attention key
    // Allow administration of the random device
    // Allow examination and configuration of disk quotas
    // Allow setting the domainname
    // Allow setting the hostname
    // Allow calling bdflush()
    // Allow mount() and umount(), setting up new smb connection
    // Allow some autofs root ioctls
    // Allow nfsservctl
    // Allow VM86_REQUEST_IRQ
    // Allow to read/write pci config on alpha
    // Allow irix_prctl on mips (setstacksize)
    // Allow flushing all cache on m68k (sys_cacheflush)
    // Allow removing semaphores
    // Used instead of CAP_CHOWN to "chown" IPC message queues, semaphores
    // and shared memory
    // Allow locking/unlocking of shared memory segment
    // Allow turning swap on/off
    // Allow forged pids on socket credentials passing
    // Allow setting readahead and flushing buffers on block devices
    // Allow setting geometry in floppy driver
    // Allow turning DMA on/off in xd driver
    // Allow administration of md devices (mostly the above, but some
    // extra ioctls)
    // Allow tuning the ide driver
    // Allow access to the nvram device
    // Allow administration of apm_bios, serial and bttv (TV) device
    // Allow manufacturer commands in isdn CAPI support driver
    // Allow reading non-standardized portions of pci configuration space
    // Allow DDI debug ioctl on sbpcd driver
    // Allow setting up serial ports
    // Allow sending raw qic-117 commands
    // Allow enabling/disabling tagged queuing on SCSI controllers and sending
    // arbitrary SCSI commands
    // Allow setting encryption key on loopback filesystem
    // Allow setting zone reclaim policy
    // Allow everything under CAP_BPF and CAP_PERFMON for backward compatibility
    SysAdmin,

    #[strum(serialize = "SYS_BOOT", serialize = "CAP_SYS_BOOT")]
    // Allow use of reboot()
    SysBoot,

    #[strum(serialize = "SYS_NICE", serialize = "CAP_SYS_NICE")]
    // Allow raising priority and setting priority on other (different
    // UID) processes
    // Allow use of FIFO and round-robin (realtime) scheduling on own
    // processes and setting the scheduling algorithm used by another
    // process.
    // Allow setting cpu affinity on other processes
    SysNice,

    // Override resource limits. Set resource limits.
    // Override quota limits.
    // Override reserved space on ext2 filesystem
    // Modify data journaling mode on ext3 filesystem (uses journaling
    // resources)
    // NOTE: ext2 honors fsuid when checking for resource overrides, so you can
    // override using fsuid too
    // Override size restrictions on IPC message queues
    // Allow more than 64hz interrupts from the real-time clock
    // Override max number of consoles on console allocation
    // Override max number of keymaps
    // Control memory reclaim behavior
    #[strum(serialize = "SYS_RESOURCE", serialize = "CAP_SYS_RESOURCE")]
    SysResource,

    // Allow manipulation of system clock
    // Allow irix_stime on mips
    // Allow setting the real-time clock
    #[strum(serialize = "SYS_TIME", serialize = "CAP_SYS_TIME")]
    SysTime,

    // Allow configuration of tty devices
    // Allow vhangup() of tty
    #[strum(serialize = "SYS_TTY_CONFIG", serialize = "CAP_SYS_TTY_CONFIG")]
    SysTtyConfig,

    #[strum(serialize = "MKNOD", serialize = "CAP_MKNOD")]
    // Allow the privileged aspects of mknod()
    Mknod,

    #[strum(serialize = "LEASE", serialize = "CAP_LEASE")]
    // Allow taking of leases on files
    Lease,

    #[strum(serialize = "AUDIT_WRITE", serialize = "CAP_AUDIT_WRITE")]
    /// Write records to kernel auditing log (since Linux 2.6.11).
    AuditWrite,

    #[strum(serialize = "AUDIT_CONTROL", serialize = "CAP_AUDIT_CONTROL")]
    /// Enable and disable kernel auditing; change auditing filter rules; retrieve auditing status
    /// and filtering rules (since Linux 2.6.11).
    AuditControl,

    #[strum(serialize = "SETFCAP", serialize = "CAP_SETFCAP")]
    /// Set arbitrary capabilities on a file (since Linux 2.6.24).
    Setfcap,

    #[strum(serialize = "MAC_OVERRIDE", serialize = "CAP_MAC_OVERRIDE")]
    // Override MAC access.
    // The base kernel enforces no MAC policy.
    // An LSM may enforce a MAC policy, and if it does and it chooses
    // to implement capability based overrides of that policy, this is
    // the capability it should use to do so.
    MacOverride,

    #[strum(serialize = "MAC_ADMIN", serialize = "CAP_MAC_ADMIN")]
    // Allow MAC configuration or state changes.
    // The base kernel requires no MAC configuration.
    // An LSM may enforce a MAC policy, and if it does and it chooses
    // to implement capability based checks on modifications to that
    // policy or the data required to maintain it, this is the
    // capability it should use to do so.
    MacAdmin,

    #[strum(serialize = "SYSLOG", serialize = "CAP_SYSLOG")]
    // Allow configuring the kernel's syslog (printk behaviour)
    Syslog,

    #[strum(serialize = "WAKE_ALARM", serialize = "CAP_WAKE_ALARM")]
    // Allow triggering something that will wake the system
    WakeAlarm,

    #[strum(serialize = "BLOCK_SUSPEND", serialize = "CAP_BLOCK_SUSPEND")]
    // Allow preventing system suspends
    BlockSuspend,

    #[strum(serialize = "AUDIT_READ", serialize = "CAP_AUDIT_READ")]
    // Allow reading the audit log via multicast netlink socket
    AuditRead,

    #[strum(serialize = "PERFMON", serialize = "CAP_PERFMON")]
    // Allow system performance and observability privileged operations
    // using perf_events, i915_perf and other kernel subsystems
    Perfmon,

    #[strum(serialize = "BPF", serialize = "CAP_BPF")]
    // CAP_BPF allows the following BPF operations:
    // - Creating all types of BPF maps
    // - Advanced verifier features
    //   - Indirect variable access
    //   - Bounded loops
    //   - BPF to BPF function calls
    //   - Scalar precision tracking
    //   - Larger complexity limits
    //   - Dead code elimination
    //   - And potentially other features
    // - Loading BPF Type Format (BTF) data
    // - Retrieve xlated and JITed code of BPF programs
    // - Use bpf_spin_lock() helper
    //
    // CAP_PERFMON relaxes the verifier checks further:
    // - BPF progs can use of pointer-to-integer conversions
    // - speculation attack hardening measures are bypassed
    // - bpf_probe_read to read arbitrary kernel memory is allowed
    // - bpf_trace_printk to print kernel memory is allowed
    //
    // CAP_SYS_ADMIN is required to use bpf_probe_write_user.
    //
    // CAP_SYS_ADMIN is required to iterate system wide loaded
    // programs, maps, links, BTFs and convert their IDs to file descriptors.
    //
    // CAP_PERFMON and CAP_BPF are required to load tracing programs.
    // CAP_NET_ADMIN and CAP_BPF are required to load networking programs.
    Bpf,

    #[strum(serialize = "CHECKPOINT_RESTORE", serialize = "CAP_CHECKPOINT_RESTORE")]
    // Allow checkpoint/restore related operations.
    // Introduced in kernel 5.9
    CheckpointRestore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::str::FromStr;

    #[test]
    fn as_ref() {
        assert_eq!(Capability::SysAdmin.as_ref(), "CAP_SYS_ADMIN");
        assert_eq!(Capability::Chown.as_ref(), "CAP_CHOWN");
        assert_eq!(Capability::Setgid.as_ref(), "CAP_SETGID");
        assert_eq!(Capability::Bpf.as_ref(), "CAP_BPF");
    }

    #[test]
    fn to_string() {
        assert_eq!(Capability::MacAdmin.to_string(), "CAP_MAC_ADMIN");
        assert_eq!(Capability::MacOverride.to_string(), "CAP_MAC_OVERRIDE");
        assert_eq!(Capability::SysTtyConfig.to_string(), "CAP_SYS_TTY_CONFIG");
        assert_eq!(Capability::SysTime.to_string(), "CAP_SYS_TIME");
    }

    #[test]
    fn from_str() -> Result<()> {
        assert_eq!(
            Capability::AuditRead,
            Capability::from_str("CAP_AUDIT_READ")?
        );
        assert_eq!(Capability::AuditRead, Capability::from_str("AUDIT_READ")?);
        assert_eq!(Capability::SysNice, Capability::from_str("CAP_SYS_NICE")?);
        assert_eq!(Capability::SysNice, Capability::from_str("SYS_NICE")?);
        assert_eq!(
            Capability::CheckpointRestore,
            Capability::from_str("CAP_CHECKPOINT_RESTORE")?
        );
        assert_eq!(
            Capability::CheckpointRestore,
            Capability::from_str("CHECKPOINT_RESTORE")?
        );
        assert_eq!(Capability::Bpf, Capability::from_str("CAP_BPF")?);
        assert_eq!(Capability::Bpf, Capability::from_str("BPF")?);
        assert!(Capability::from_str("wrong").is_err());
        Ok(())
    }

    #[test]
    fn into_static_str() {
        let cap: &'static str = Capability::Fowner.into();
        assert_eq!(cap, "CAP_FOWNER");
    }

    #[test]
    fn iter() {
        assert_eq!(Capability::iter().count(), 41);
    }

    #[test]
    fn all() {
        assert_eq!(Capabilities::all().0.len(), 41);
    }
}
