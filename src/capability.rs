//! Linux capability handling

use lazy_static::lazy_static;
use std::string::ToString;
use std::{collections::HashSet, ops::Deref};
use strum::{AsRefStr, Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Clone, Debug)]
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

impl Default for Capabilities {
    fn default() -> Self {
        DEFAULT_CAPABILITIES.clone()
    }
}

impl Into<Vec<String>> for Capabilities {
    fn into(self) -> Vec<String> {
        (&self).into()
    }
}

impl Into<Vec<String>> for &Capabilities {
    fn into(self) -> Vec<String> {
        self.iter().map(ToString::to_string).collect()
    }
}

lazy_static! {
    static ref DEFAULT_CAPABILITIES: Capabilities = {
        let mut s = HashSet::new();
        s.insert(Capability::CapChown);
        s.insert(Capability::CapDacOverride);
        s.insert(Capability::CapFsetid);
        s.insert(Capability::CapFowner);
        s.insert(Capability::CapSetgid);
        s.insert(Capability::CapSetuid);
        s.insert(Capability::CapSetpcap);
        s.insert(Capability::CapNetBindService);
        s.insert(Capability::CapKill);
        Capabilities(s)
    };
}

#[derive(
    AsRefStr, IntoStaticStr, Copy, Clone, Debug, Display, EnumIter, EnumString, Eq, Hash, PartialEq,
)]
#[strum(serialize_all = "shouty_snake_case")]
/// All available capabilities.
pub enum Capability {
    // In a system with the [_POSIX_CHOWN_RESTRICTED] option defined, this
    // overrides the restriction of changing file ownership and group ownership.
    CapChown,

    // Override all DAC access, including ACL execute access if [_POSIX_ACL] is
    // defined. Excluding DAC access covered by CAP_LINUX_IMMUTABLE.
    CapDacOverride,

    // Overrides all DAC restrictions regarding read and search on files
    // and directories, including ACL restrictions if [_POSIX_ACL] is
    // defined. Excluding DAC access covered by CAP_LINUX_IMMUTABLE.
    CapDacReadSearch,

    // Overrides all restrictions about allowed operations on files, where
    // file owner ID must be equal to the user ID, except where CAP_FSETID
    // is applicable. It doesn't override MAC and DAC restrictions.
    CapFowner,

    // Overrides the following restrictions that the effective user ID
    // shall match the file owner ID when setting the S_ISUID and S_ISGID
    // bits on that file; that the effective group ID (or one of the
    // supplementary group IDs) shall match the file owner ID when setting
    // the S_ISGID bit on that file; that the S_ISUID and S_ISGID bits are
    // cleared on successful return from chown(2) (not implemented).
    CapFsetid,

    // Overrides the restriction that the real or effective user ID of a
    // process sending a signal must match the real or effective user ID
    // of the process receiving the signal.
    CapKill,

    // Allows setgid(2) manipulation
    // Allows setgroups(2)
    // Allows forged gids on socket credentials passing.
    CapSetgid,

    // Allows set*uid(2) manipulation (including fsuid).
    // Allows forged pids on socket credentials passing.
    CapSetuid,

    // Without VFS support for capabilities:
    //   Transfer any capability in your permitted set to any pid,
    //   remove any capability in your permitted set from any pid
    // With VFS support for capabilities (neither of above, but)
    //   Add any capability from current's capability bounding set
    //     to the current process' inheritable set
    //   Allow taking bits out of capability bounding set
    //   Allow modification of the securebits for a process
    CapSetpcap,

    // Allow modification of S_IMMUTABLE and S_APPEND file attributes
    CapLinuxImmutable,

    // Allows binding to TCP/UDP sockets below 1024
    // Allows binding to ATM VCIs below 32
    CapNetBindService,

    // Allow broadcasting, listen to multicast
    CapNetBroadcast,

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
    CapNetAdmin,

    // Allow use of RAW sockets
    // Allow use of PACKET sockets
    // Allow binding to any address for transparent proxying (also via
    // NET_ADMIN)
    CapNetRaw,

    // Allow locking of shared memory segments
    // Allow mlock and mlockall (which doesn't really have anything to do
    // with IPC)
    CapIpcLock,

    // Override IPC ownership checks
    CapIpcOwner,

    // Insert and remove kernel modules - modify kernel without limit
    CapSysModule,

    // Allow ioperm/iopl access
    // Allow sending USB messages to any device via /proc/bus/usb
    CapSysRawio,

    // Allow use of chroot()
    CapSysChroot,

    // Allow ptrace() of any process
    CapSysPtrace,

    // Allow configuration of process accounting
    CapSysPacct,

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
    CapSysAdmin,

    // Allow use of reboot()
    CapSysBoot,

    // Allow raising priority and setting priority on other (different
    // UID) processes
    // Allow use of FIFO and round-robin (realtime) scheduling on own
    // processes and setting the scheduling algorithm used by another
    // process.
    // Allow setting cpu affinity on other processes
    CapSysNice,

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
    CapSysResource,

    // Allow manipulation of system clock
    // Allow irix_stime on mips
    // Allow setting the real-time clock
    CapSysTime,

    // Allow configuration of tty devices
    // Allow vhangup() of tty
    CapSysTtyConfig,

    // Allow the privileged aspects of mknod()
    CapMknod,

    // Allow taking of leases on files
    CapLease,

    CapAuditWrite,
    CapAuditControl,
    CapSetfcap,

    // Override MAC access.
    // The base kernel enforces no MAC policy.
    // An LSM may enforce a MAC policy, and if it does and it chooses
    // to implement capability based overrides of that policy, this is
    // the capability it should use to do so.
    CapMacOverride,

    // Allow MAC configuration or state changes.
    // The base kernel requires no MAC configuration.
    // An LSM may enforce a MAC policy, and if it does and it chooses
    // to implement capability based checks on modifications to that
    // policy or the data required to maintain it, this is the
    // capability it should use to do so.
    CapMacAdmin,

    // Allow configuring the kernel's syslog (printk behaviour)
    CapSyslog,

    // Allow triggering something that will wake the system
    CapWakeAlarm,

    // Allow preventing system suspends
    CapBlockSuspend,

    // Allow reading the audit log via multicast netlink socket
    CapAuditRead,

    // Allow system performance and observability privileged operations
    // using perf_events, i915_perf and other kernel subsystems
    CapPerfmon,

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
    CapBpf,

    // Allow checkpoint/restore related operations.
    // Introduced in kernel 5.9
    CapCheckpointRestore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::str::FromStr;

    #[test]
    fn as_ref() {
        assert_eq!(Capability::CapSysAdmin.as_ref(), "CAP_SYS_ADMIN");
        assert_eq!(Capability::CapChown.as_ref(), "CAP_CHOWN");
        assert_eq!(Capability::CapSetgid.as_ref(), "CAP_SETGID");
    }

    #[test]
    fn from_str() -> Result<()> {
        assert_eq!(
            Capability::CapAuditRead,
            Capability::from_str("CAP_AUDIT_READ")?
        );
        assert_eq!(
            Capability::CapSysNice,
            Capability::from_str("CAP_SYS_NICE")?
        );
        assert_eq!(
            Capability::CapCheckpointRestore,
            Capability::from_str("CAP_CHECKPOINT_RESTORE")?
        );
        Ok(())
    }

    #[test]
    fn into_static_str() {
        let cap: &'static str = Capability::CapFowner.into();
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
