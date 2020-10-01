//! OCI runtime spec

use anyhow::{Context, Result};
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// Spec is the base configuration for the container.
pub struct Spec {
    #[getset(get = "pub")]
    #[serde(rename = "ociVersion")]
    /// Version of the Open Container Initiative Runtime Specification with which the bundle
    /// complies.
    version: String,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Process configures the container process.
    process: Option<Process>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Root configures the container's root filesystem.
    root: Option<Root>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Hostname configures the container's hostname.
    hostname: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Mounts configures additional mounts (on top of Root).
    mounts: Option<Vec<Mount>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Hooks configures callbacks for container lifecycle events.
    hooks: Option<Hooks>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Annotations contains arbitrary metadata for the container.
    annotations: Option<HashMap<String, String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Linux is platform-specific configuration for Linux based containers.
    linux: Option<Linux>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Solaris is platform-specific configuration for Solaris based containers.
    solaris: Option<Solaris>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Windows is platform-specific configuration for Windows based containers.
    windows: Option<Windows>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// VM specifies configuration for virtual-machine-based containers.
    vm: Option<VM>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            process: None,
            root: None,
            hostname: None,
            mounts: None,
            hooks: None,
            annotations: None,
            linux: None,
            solaris: None,
            windows: None,
            vm: None,
        }
    }
}

impl Spec {
    #[allow(dead_code)]
    /// Load a new spec from the provided file `Path`
    pub fn from(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("open file {}", path.display()))?;
        serde_json::from_reader(file)
            .with_context(|| format!("deserialize OCI spec from file {}", path.display()))
    }

    #[allow(dead_code)]
    /// Save the loaded spec into the provided file `Path`
    pub fn save(&self, path: &Path) -> Result<()> {
        let mut file =
            File::create(path).with_context(|| format!("create file {}", path.display()))?;
        serde_json::to_writer(&mut file, self)
            .with_context(|| format!("serialize OCI spec to file {}", path.display()))
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// Process contains information to start a specific application inside the container.
pub struct Process {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Terminal creates an interactive terminal for the container.
    terminal: Option<bool>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "consoleSize"
    )]
    /// ConsoleSize specifies the size of the console.
    console_size: Option<Box>,

    /// User specifies user information for the process.
    #[getset(get = "pub")]
    user: User,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Args specifies the binary and arguments for the application to execute.
    args: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "commandLine"
    )]
    /// CommandLine specifies the full command line for the application to execute on Windows.
    command_line: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Env populates the process environment for the process.
    env: Option<Vec<String>>,

    #[getset(get = "pub")]
    /// Cwd is the current working directory for the process and must be relative to the
    /// container's root.
    cwd: String,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Capabilities are Linux capabilities that are kept for the process.
    capabilities: Option<LinuxCapabilities>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Rlimits specifies rlimit options to apply to the process.
    rlimits: Option<Vec<POSIXRlimit>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "noNewPrivileges"
    )]
    /// NoNewPrivileges controls whether additional privileges could be gained by processes in the
    /// container.
    no_new_privileges: Option<bool>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "apparmorProfile"
    )]
    /// ApparmorProfile specifies the apparmor profile for the container.
    apparmor_profile: Option<String>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "oomScoreAdj")]
    /// Specify an oom_score_adj for the container.
    oom_score_adj: Option<i32>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "selinuxLabel"
    )]
    /// SelinuxLabel specifies the selinux context that the container process is run as.
    selinux_label: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxCapabilities specifies the list of allowed capabilities that are kept for a process.
/// http://man7.org/linux/man-pages/man7/capabilities.7.html
pub struct LinuxCapabilities {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Bounding is the set of capabilities checked by the kernel.
    bounding: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Effective is the set of capabilities checked by the kernel.
    effective: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Inheritable is the capabilities preserved across execve.
    inheritable: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Permitted is the limiting superset for effective capabilities.
    permitted: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    //// Ambient is the ambient set of capabilities that are kept.
    ambient: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Default, Serialize, Deserialize, Debug, Builder, CopyGetters)]
#[builder(default, pattern = "owned", setter(into))]
/// Box specifies dimensions of a rectangle. Used for specifying the size of a console.
pub struct Box {
    #[getset(get_copy = "pub")]
    /// Height is the vertical dimension of a box.
    height: u64,

    #[getset(get_copy = "pub")]
    /// Width is the horizontal dimension of a box.
    width: u64,
}

/// User specifies specific user (and group) information for the container process.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct User {
    #[getset(get_copy = "pub")]
    /// UID is the user id.
    uid: u32,

    #[getset(get_copy = "pub")]
    /// GID is the group id.
    gid: u32,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Umask is the umask for the init process.
    umask: Option<u32>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "additionalGids"
    )]
    /// AdditionalGids are additional group ids set for the container's process.
    additional_gids: Option<Vec<u32>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Username is the user name.
    username: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// Root contains information about the container's root filesystem on the host.
pub struct Root {
    #[getset(get = "pub")]
    /// Path is the absolute path to the container's root filesystem.
    path: PathBuf,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Readonly makes the root filesystem for the container readonly before the process is
    /// executed.
    readonly: Option<bool>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// Mount specifies a mount for a container.
pub struct Mount {
    #[getset(get = "pub")]
    /// Destination is the absolute path where the mount will be placed in the container.
    destination: PathBuf,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    /// Type specifies the mount kind.
    typ: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Source specifies the source path of the mount.
    source: Option<PathBuf>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Options are fstab style mount options.
    options: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// Hook specifies a command that is run at a particular event in the lifecycle of a container.
pub struct Hook {
    #[getset(get = "pub")]
    path: PathBuf,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    env: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout: Option<i64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// Hooks specifies a command that is run in the container at a particular event in the lifecycle
/// (setup and teardown) of a container.
pub struct Hooks {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Prestart is Deprecated. Prestart is a list of hooks to be run before the container process
    /// is executed. It is called in the Runtime Namespace
    prestart: Option<Vec<Hook>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "createRuntime"
    )]
    /// CreateRuntime is a list of hooks to be run after the container has been created but before
    /// pivot_root or any equivalent operation has been called. It is called in the Runtime
    /// Namespace.
    create_runtime: Option<Vec<Hook>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "createContainer"
    )]
    /// CreateContainer is a list of hooks to be run after the container has been created but
    /// before pivot_root or any equivalent operation has been called. It is called in the
    /// Container Namespace.
    create_container: Option<Vec<Hook>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "startContainer"
    )]
    /// StartContainer is a list of hooks to be run after the start operation is called but before
    /// the container process is started. It is called in the Container Namespace.
    start_container: Option<Vec<Hook>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Poststart is a list of hooks to be run after the container process is started. It is called
    /// in the Runtime Namespace.
    poststart: Option<Vec<Hook>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Poststop is a list of hooks to be run after the container process exits. It is called in
    /// the Runtime Namespace.
    poststop: Option<Vec<Hook>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// Linux contains platform-specific configuration for Linux based containers.
pub struct Linux {
    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "uidMappings"
    )]
    /// UIDMappings specifies user mappings for supporting user namespaces.
    uid_mappings: Option<Vec<LinuxIDMapping>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "gidMappings"
    )]
    /// GIDMappings specifies group mappings for supporting user namespaces.
    gid_mappings: Option<Vec<LinuxIDMapping>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Sysctl are a set of key value pairs that are set for the container on start.
    sysctl: Option<HashMap<String, String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Resources contain cgroup information for handling resource constraints for the container.
    resources: Option<LinuxResources>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "cgroupsPath"
    )]
    /// CgroupsPath specifies the path to cgroups that are created and/or joined by the container.
    /// The path is expected to be relative to the cgroups mountpoint. If resources are specified,
    /// the cgroups at CgroupsPath will be updated based on resources.
    cgroups_path: Option<PathBuf>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Namespaces contains the namespaces that are created and/or joined by the container.
    namespaces: Option<Vec<LinuxNamespace>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Devices are a list of device nodes that are created for the container.
    devices: Option<Vec<LinuxDevice>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Seccomp specifies the seccomp security settings for the container.
    seccomp: Option<LinuxSeccomp>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "rootfsPropagation"
    )]
    /// RootfsPropagation is the rootfs mount propagation mode for the container.
    rootfs_propagation: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "maskedPaths"
    )]
    /// MaskedPaths masks over the provided paths inside the container.
    masked_paths: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "readonlyPaths"
    )]
    /// ReadonlyPaths sets the provided paths as RO inside the container.
    readonly_paths: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "mountLabel"
    )]
    /// MountLabel specifies the selinux context for the mounts in the container.
    mount_label: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "intelRdt")]
    /// IntelRdt contains Intel Resource Director Technology (RDT) information for handling
    /// resource constraints (e.g., L3 cache, memory bandwidth) for the container.
    intel_rdt: Option<LinuxIntelRdt>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Personality contains configuration for the Linux personality syscall.
    personality: Option<LinuxPersonality>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxNamespace is the configuration for a Linux namespace.
pub struct LinuxNamespace {
    #[getset(get_copy = "pub")]
    #[serde(rename = "type")]
    /// Type is the type of namespace.
    typ: LinuxNamespaceType,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Path is a path to an existing namespace persisted on disk that can be joined and is of the
    /// same type
    path: Option<PathBuf>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LinuxNamespaceType {
    #[serde(rename = "pid")]
    /// For isolating process IDs.
    Pid,

    #[serde(rename = "network")]
    /// For isolating network devices, stacks, ports, etc..
    Network,

    #[serde(rename = "mount")]
    /// For isolating mount points.
    Mount,

    #[serde(rename = "ipc")]
    /// For isolating System V IPC, POSIX message queues.
    Ipc,

    #[serde(rename = "utc")]
    /// For isolating hostname and NIS domain name.
    Uts,

    #[serde(rename = "user")]
    /// For isolating user and group IDs.
    User,

    #[serde(rename = "cgroup")]
    /// For isolating cgroup hierarchies.
    Cgroup,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxIDMapping specifies UID/GID mappings.
pub struct LinuxIDMapping {
    #[getset(get_copy = "pub")]
    #[serde(rename = "containerID")]
    /// ContainerID is the starting UID/GID in the container.
    container_id: u32,

    #[getset(get_copy = "pub")]
    #[serde(rename = "hostID")]
    /// HostID is the starting UID/GID on the host to be mapped to `container_id`.
    host_id: u32,

    #[getset(get_copy = "pub")]
    /// Size is the number of IDs to be mapped.
    size: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// POSIXRlimit type and restrictions.
pub struct POSIXRlimit {
    #[getset(get = "pub")]
    #[serde(rename = "type")]
    /// Type of the rlimit to set.
    typ: String,

    #[getset(get_copy = "pub")]
    /// Hard is the hard limit for the specified type.
    hard: u64,

    #[getset(get_copy = "pub")]
    /// Soft is the soft limit for the specified type.
    soft: u64,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxHugepageLimit structure corresponds to limiting kernel hugepages.
pub struct LinuxHugepageLimit {
    #[getset(get = "pub")]
    #[serde(rename = "pageSize")]
    /// Pagesize is the hugepage size.
    /// Format: "<size><unit-prefix>B' (e.g. 64KB, 2MB, 1GB, etc.)
    page_size: String,

    #[getset(get_copy = "pub")]
    /// Limit is the limit of "hugepagesize" hugetlb usage.
    limit: i64,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxInterfacePriority for network interfaces.
pub struct LinuxInterfacePriority {
    #[getset(get = "pub")]
    /// Name is the name of the network interface.
    name: String,

    #[getset(get_copy = "pub")]
    /// Priority for the interface.
    priority: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxWeightDevice struct holds a `major:minor weight` pair for weightDevice.
pub struct LinuxWeightDevice {
    #[getset(get_copy = "pub")]
    /// Major is the device's major number.
    major: i64,

    #[getset(get_copy = "pub")]
    /// Minor is the device's minor number.
    minor: i64,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Weight is the bandwidth rate for the device.
    weight: Option<u16>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "leafWeight")]
    /// LeafWeight is the bandwidth rate for the device while competing with the cgroup's child
    /// cgroups, CFQ scheduler only.
    leaf_weight: Option<u16>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct LinuxThrottleDevice {
    #[getset(get_copy = "pub")]
    /// Major is the device's major number.
    major: i64,

    #[getset(get_copy = "pub")]
    /// Minor is the device's minor number.
    minor: i64,

    #[getset(get_copy = "pub")]
    /// Rate is the IO rate limit per cgroup per device.
    rate: u64,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxBlockIO for Linux cgroup 'blkio' resource management.
pub struct LinuxBlockIO {
    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Specifies per cgroup weight.
    weight: Option<u16>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "leafWeight")]
    /// Specifies tasks' weight in the given cgroup while competing with the cgroup's child
    /// cgroups, CFQ scheduler only.
    leaf_weight: Option<u16>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "weightDevice"
    )]
    /// Weight per cgroup per device, can override BlkioWeight.
    weight_device: Option<Vec<LinuxWeightDevice>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "throttleReadBpsDevice"
    )]
    /// IO read rate limit per cgroup per device, bytes per second.
    throttle_read_bps_device: Option<Vec<LinuxThrottleDevice>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "throttleWriteBpsDevice"
    )]
    /// IO write rate limit per cgroup per device, bytes per second.
    throttle_write_bps_device: Option<Vec<LinuxThrottleDevice>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "throttleReadIOPSDevice"
    )]
    /// IO read rate limit per cgroup per device, IO per second.
    throttle_read_iops_device: Option<Vec<LinuxThrottleDevice>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "throttleWriteIOPSDevice"
    )]
    /// IO write rate limit per cgroup per device, IO per second.
    throttle_write_iops_device: Option<Vec<LinuxThrottleDevice>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxMemory for Linux cgroup 'memory' resource management.
pub struct LinuxMemory {
    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Memory limit (in bytes).
    limit: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Memory reservation or soft_limit (in bytes).
    reservation: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Total memory limit (memory + swap).
    swap: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Kernel memory limit (in bytes).
    kernel: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "kernelTCP")]
    /// Kernel memory limit for tcp (in bytes).
    kernel_tcp: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// How aggressive the kernel will swap memory pages.
    swappiness: Option<u64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "disableOOMKiller")]
    /// DisableOOMKiller disables the OOM killer for out of memory conditions.
    disable_oom_killer: Option<bool>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "useHierarchy")]
    /// Enables hierarchical memory accounting
    use_hierarchy: Option<bool>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxCPU for Linux cgroup 'cpu' resource management.
pub struct LinuxCPU {
    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// CPU shares (relative weight (ratio) vs. other cgroups with cpu shares).
    shares: Option<u64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// CPU hardcap limit (in usecs). Allowed cpu time in a given period.
    quota: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// CPU period to be used for hardcapping (in usecs).
    period: Option<u64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "realtimeRuntime")]
    /// How much time realtime scheduling may use (in usecs).
    realtime_runtime: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "realtimePeriod")]
    /// CPU period to be used for realtime scheduling (in usecs).
    realtime_period: Option<u64>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// CPUs to use within the cpuset. Default is to use any CPU available.
    cpus: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// List of memory nodes in the cpuset. Default is to use any available memory node.
    mems: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxPids for Linux cgroup 'pids' resource management (Linux 4.3).
pub struct LinuxPids {
    #[getset(get_copy = "pub")]
    /// Maximum number of PIDs. Default is "no limit".
    limit: i64,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxNetwork identification and priority configuration.
pub struct LinuxNetwork {
    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "classID")]
    /// Set class identifier for container's network packets
    class_id: Option<u32>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set priority of network traffic for container.
    priorities: Option<Vec<LinuxInterfacePriority>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxRdma for Linux cgroup 'rdma' resource management (Linux 4.11).
pub struct LinuxRdma {
    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "hcaHandles")]
    /// Maximum number of HCA handles that can be opened. Default is "no limit".
    hca_handles: Option<u32>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "hcaObjects")]
    /// Maximum number of HCA objects that can be created. Default is "no limit".
    hca_objects: Option<u32>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxResources has container runtime resource constraints.
pub struct LinuxResources {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Devices configures the device allowlist.
    devices: Option<Vec<LinuxDeviceCgroup>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Memory restriction configuration.
    memory: Option<LinuxMemory>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// CPU resource restriction configuration.
    cpu: Option<LinuxCPU>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Task resource restriction configuration.
    pids: Option<LinuxPids>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "blockIO")]
    /// BlockIO restriction configuration.
    block_io: Option<LinuxBlockIO>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "hugepageLimits"
    )]
    /// Hugetlb limit (in bytes).
    hugepage_limits: Option<Vec<LinuxHugepageLimit>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Network restriction configuration.
    network: Option<LinuxNetwork>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Rdma resource restriction configuration. Limits are a set of key value pairs that define
    /// RDMA resource limits, where the key is device name and value is resource limits.
    rdma: Option<HashMap<String, LinuxRdma>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Unified resources.
    unified: Option<HashMap<String, String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxDevice represents the mknod information for a Linux special device file.
pub struct LinuxDevice {
    #[getset(get = "pub")]
    /// Path to the device.
    path: PathBuf,

    #[getset(get = "pub")]
    #[serde(rename = "type")]
    /// Device type, block, char, etc..
    typ: String,

    #[getset(get_copy = "pub")]
    /// Major is the device's major number.
    major: i64,

    #[getset(get_copy = "pub")]
    /// Minor is the device's minor number.
    minor: i64,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "fileMode")]
    /// FileMode permission bits for the device.
    file_mode: Option<u32>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// UID of the device.
    uid: Option<u32>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Gid of the device.
    gid: Option<u32>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxDeviceCgroup represents a device rule for the devices specified to the device controller.
pub struct LinuxDeviceCgroup {
    #[getset(get_copy = "pub")]
    /// Allow or deny.
    allow: bool,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    /// Device type, block, char, etc..
    typ: Option<String>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Major is the device's major number.
    major: Option<i64>,

    #[getset(get = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Minor is the device's minor number.
    minor: Option<i64>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Cgroup access permissions format, rwm.
    access: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// LinuxPersonality represents the Linux personality syscall input.
pub struct LinuxPersonality {
    #[getset(get_copy = "pub")]
    /// Domain for the personality.
    domain: LinuxPersonalityDomain,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Additional flags
    flags: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Copy)]
/// Define domain and flags for LinuxPersonality.
pub enum LinuxPersonalityDomain {
    #[serde(rename = "LINUX")]
    /// PerLinux is the standard Linux personality.
    PerLinux,

    #[serde(rename = "LINUX32")]
    /// PerLinux32 sets personality to 32 bit.
    PerLinux32,
}

#[derive(Default, PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxSeccomp represents syscall restrictions.
pub struct LinuxSeccomp {
    #[getset(get_copy = "pub")]
    #[serde(rename = "defaultAction")]
    default_action: LinuxSeccompAction,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    architectures: Option<Vec<Arch>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flags: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    syscalls: Option<Vec<LinuxSyscall>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum LinuxSeccompAction {
    #[serde(rename = "SCMP_ACT_KILL")]
    Kill,

    #[serde(rename = "SCMP_ACT_KILL_PROCESS")]
    KillProcess,

    #[serde(rename = "SCMP_ACT_TRAP")]
    Trap,

    #[serde(rename = "SCMP_ACT_ERRNO")]
    Errno,

    #[serde(rename = "SCMP_ACT_TRACE")]
    Trace,

    #[serde(rename = "SCMP_ACT_ALLOW")]
    Allow,

    #[serde(rename = "SCMP_ACT_LOG")]
    Log,
}

impl Default for LinuxSeccompAction {
    fn default() -> Self {
        LinuxSeccompAction::Allow
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Arch {
    #[serde(rename = "SCMP_ARCH_X86")]
    X86,

    #[serde(rename = "SCMP_ARCH_X86_64")]
    X86_64,

    #[serde(rename = "SCMP_ARCH_X32")]
    X32,

    #[serde(rename = "SCMP_ARCH_ARM")]
    ARM,

    #[serde(rename = "SCMP_ARCH_AARCH64")]
    AARCH64,

    #[serde(rename = "SCMP_ARCH_MIPS")]
    MIPS,

    #[serde(rename = "SCMP_ARCH_MIPS64")]
    MIPS64,

    #[serde(rename = "SCMP_ARCH_MIPS64N32")]
    MIPS64N32,

    #[serde(rename = "SCMP_ARCH_MIPSEL")]
    MIPSEL,

    #[serde(rename = "SCMP_ARCH_MIPSEL64")]
    MIPSEL64,

    #[serde(rename = "SCMP_ARCH_MIPSEL64N32")]
    MIPSEL64N32,

    #[serde(rename = "SCMP_ARCH_PPC")]
    PPC,

    #[serde(rename = "SCMP_ARCH_PPC64")]
    PPC64,

    #[serde(rename = "SCMP_ARCH_PPC64LE")]
    PPC64LE,

    #[serde(rename = "SCMP_ARCH_S390")]
    S390,

    #[serde(rename = "SCMP_ARCH_S390X")]
    S390X,

    #[serde(rename = "SCMP_ARCH_PARISC")]
    PARISC,

    #[serde(rename = "SCMP_ARCH_PARISC64")]
    PARISC64,

    #[serde(rename = "SCMP_ARCH_RISCV64")]
    RISCV64,
}

#[derive(Default, PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxSyscall is used to match a syscall in seccomp.
pub struct LinuxSyscall {
    #[getset(get = "pub")]
    names: Vec<String>,

    #[getset(get_copy = "pub")]
    action: LinuxSeccompAction,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    errno_ret: Option<u32>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<Vec<LinuxSeccompArg>>,
}

#[derive(Default, PartialEq, Eq, Serialize, Deserialize, Debug, Builder, CopyGetters, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxSeccompArg used for matching specific syscall arguments in seccomp.
pub struct LinuxSeccompArg {
    #[getset(get_copy = "pub")]
    index: usize,

    #[getset(get_copy = "pub")]
    value: u64,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "valueTwo")]
    value_two: Option<u64>,

    #[getset(get_copy = "pub")]
    op: LinuxSeccompOperator,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, Copy)]
/// The seccomp operator to be used for args.
pub enum LinuxSeccompOperator {
    #[serde(rename = "SCMP_CMP_NE")]
    /// Refers to the SCMP_CMP_NE operator
    NotEqual,

    #[serde(rename = "SCMP_CMP_LT")]
    /// Refers to the SCMP_CMP_LT operator
    LessThan,

    #[serde(rename = "SCMP_CMP_LE")]
    /// Refers to the SCMP_CMP_LE operator
    LessEqual,

    #[serde(rename = "SCMP_CMP_EQ")]
    /// Refers to the SCMP_CMP_EQ operator
    EqualTo,

    #[serde(rename = "SCMP_CMP_GE")]
    /// Refers to the SCMP_CMP_GE operator
    GreaterEqual,

    #[serde(rename = "SCMP_CMP_GT")]
    /// Refers to the SCMP_CMP_GT operator
    GreaterThan,

    #[serde(rename = "SCMP_CMP_MASKED_EQ")]
    /// Refers to the SCMP_CMP_MASKED_EQ operator
    MaskedEqual,
}

impl Default for LinuxSeccompOperator {
    fn default() -> Self {
        LinuxSeccompOperator::EqualTo
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// LinuxIntelRdt has container runtime resource constraints for Intel RDT CAT and MBA features
/// which introduced in Linux 4.10 and 4.12 kernel.
pub struct LinuxIntelRdt {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "closID")]
    /// The identity for RDT Class of Service.
    clos_id: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "l3CacheSchema"
    )]
    /// The schema for L3 cache id and capacity bitmask (CBM).
    /// Format: "L3:<cache_id0>=<cbm0>;<cache_id1>=<cbm1>;..."
    l3_cache_schema: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "memBwSchema"
    )]
    /// The schema of memory bandwidth per L3 cache id.
    /// Format: "MB:<cache_id0>=bandwidth0;<cache_id1>=bandwidth1;..."
    /// The unit of memory bandwidth is specified in "percentages" by default, and in "MBps" if MBA
    /// Software Controller is enabled.
    mem_bw_schema: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// Solaris contains platform-specific configuration for Solaris application containers.
pub struct Solaris {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// SMF FMRI which should go "online" before we start the container process.
    milestone: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "limitpriv")]
    /// Maximum set of privileges any process in this container can obtain.
    limit_priv: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "maxShmMemory"
    )]
    /// The maximum amount of shared memory allowed for this container.
    max_shm_memory: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Specification for automatic creation of network resources for this container.
    anet: Option<Vec<SolarisAnet>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "cappedCPU")]
    /// Set limit on the amount of CPU time that can be used by container.
    capped_cpu: Option<SolarisCappedCPU>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "cappedMemory"
    )]
    /// The physical and swap caps on the memory that can be used by this container.
    capped_memory: Option<SolarisCappedMemory>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// SolarisAnet provides the specification for automatic creation of network resources for this
/// container.
pub struct SolarisAnet {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Specify a name for the automatically created VNIC datalink.
    linkname: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lowerLink")]
    /// Specify the link over which the VNIC will be created.
    lowerlink: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "allowedAddress"
    )]
    /// The set of IP addresses that the container can use.
    allowed_address: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "configureAllowedAddress"
    )]
    /// Specifies whether allowedAddress limitation is to be applied to the VNIC.
    configure_allowed_address: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The value of the optional default router.
    defrouter: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "linkProtection"
    )]
    /// Enable one or more types of link protection.
    link_protection: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "macAddress"
    )]
    /// Set the VNIC's macAddress
    mac_address: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// SolarisCappedCPU allows users to set limit on the amount of CPU time that can be used by
/// container.
pub struct SolarisCappedCPU {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ncpus: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// SolarisCappedMemory allows users to set the physical and swap caps on the memory that can be
/// used by this container.
pub struct SolarisCappedMemory {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The physical caps on the memory.
    physical: Option<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The swap caps on the memory.
    swap: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// Windows defines the runtime configuration for Windows based containers, including Hyper-V
/// containers.
pub struct Windows {
    #[getset(get = "pub")]
    #[serde(rename = "layerFolders")]
    /// LayerFolders contains a list of absolute paths to directories containing image layers.
    layer_folders: Vec<String>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Devices are the list of devices to be mapped into the container.
    devices: Option<Vec<WindowsDevice>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Resources contains information for handling resource constraints for the container.
    resources: Option<WindowsResources>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "credentialSpec"
    )]
    /// CredentialSpec contains a JSON object describing a group Managed Service Account (gMSA)
    /// specification.
    credential_spec: Option<HashMap<String, Option<serde_json::Value>>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Servicing indicates if the container is being started in a mode to apply a Windows Update
    /// servicing operation.
    servicing: Option<bool>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "ignoreFlushesDuringBoot"
    )]
    /// IgnoreFlushesDuringBoot indicates if the container is being started in a mode where disk
    /// writes are not flushed during its boot process.
    ignore_flushes_during_boot: Option<bool>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// HyperV contains information for running a container with Hyper-V isolation.
    hyperv: Option<WindowsHyperV>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Network restriction configuration.
    network: Option<WindowsNetwork>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// WindowsDevice represents information about a host device to be mapped into the container.
pub struct WindowsDevice {
    #[getset(get = "pub")]
    /// Device identifier: interface class GUID, etc..
    id: String,

    #[getset(get = "pub")]
    #[serde(rename = "idType")]
    /// Device identifier type: "class", etc..
    id_type: String,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
pub struct WindowsResources {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Memory restriction configuration.
    memory: Option<WindowsMemoryResources>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// CPU resource restriction configuration.
    cpu: Option<WindowsCPUResources>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Storage restriction configuration.
    storage: Option<WindowsStorageResources>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// WindowsMemoryResources contains memory resource management settings.
pub struct WindowsMemoryResources {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Memory limit in bytes.
    limit: Option<u64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default, Builder, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// WindowsCPUResources contains CPU resource management settings.
pub struct WindowsCPUResources {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Number of CPUs available to the container.
    count: Option<u64>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// CPU shares (relative weight to other containers with cpu shares).
    shares: Option<u16>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Specifies the portion of processor cycles that this container can use as a percentage times
    /// 100.
    maximum: Option<u16>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Default, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// WindowsStorageResources contains storage resource management settings.
pub struct WindowsStorageResources {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Specifies maximum Iops for the system drive.
    iops: Option<u64>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Specifies maximum bytes per second for the system drive.
    bps: Option<u64>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "sandboxSize"
    )]
    /// Sandbox size specifies the minimum size of the system drive in bytes.
    sandbox_size: Option<u64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Default, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// WindowsHyperV contains information for configuring a container to run with Hyper-V isolation.
pub struct WindowsHyperV {
    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "utilityVMPath"
    )]
    /// UtilityVMPath is an optional path to the image used for the Utility VM.
    utility_vm_path: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Default, Getters)]
#[builder(default, pattern = "owned", setter(into, strip_option))]
/// WindowsNetwork contains network settings for Windows containers.
pub struct WindowsNetwork {
    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "endpointList"
    )]
    /// List of HNS endpoints that the container should connect to.
    endpoint_list: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "allowUnqualifiedDNSQuery"
    )]
    /// Specifies if unqualified DNS name resolution is allowed.
    allow_unqualified_dns_query: Option<bool>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "DNSSearchList"
    )]
    /// Comma separated list of DNS suffixes to use for name resolution.
    dns_search_list: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "networkSharedContainerName"
    )]
    /// Name (ID) of the container that we will share with the network stack.
    network_shared_container_name: Option<String>,

    #[getset(get = "pub")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "networkNamespace"
    )]
    /// name (ID) of the network namespace that will be used for the container.
    network_namespace: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// VM contains information for virtual-machine-based containers.
pub struct VM {
    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Hypervisor specifies hypervisor-related configuration for virtual-machine-based containers.
    hypervisor: Option<VMHypervisor>,

    #[getset(get = "pub")]
    /// Kernel specifies kernel-related configuration for virtual-machine-based containers.
    kernel: VMKernel,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Image specifies guest image related configuration for virtual-machine-based containers.
    image: Option<VMImage>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// VMHypervisor contains information about the hypervisor to use for a virtual machine.
pub struct VMHypervisor {
    #[getset(get = "pub")]
    /// Path is the host path to the hypervisor used to manage the virtual machine.
    path: PathBuf,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Parameters specifies parameters to pass to the hypervisor.
    parameters: Option<Vec<String>>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// VMKernel contains information about the kernel to use for a virtual machine.
pub struct VMKernel {
    #[getset(get = "pub")]
    /// Path is the host path to the kernel used to boot the virtual machine.
    path: PathBuf,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Parameters specifies parameters to pass to the kernel.
    parameters: Option<Vec<String>>,

    #[getset(get = "pub")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// InitRD is the host path to an initial ramdisk to be used by the kernel.
    initrd: Option<String>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Builder, Getters)]
#[builder(pattern = "owned", setter(into, strip_option))]
/// VMImage contains information about the virtual machine root image.
pub struct VMImage {
    #[getset(get = "pub")]
    /// Path is the host path to the root image that the VM kernel would boot into.
    path: PathBuf,

    #[getset(get = "pub")]
    /// Format is the root image format type (e.g. "qcow2", "raw", "vhd", etc).
    format: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write};
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn build_spec() -> Result<()> {
        let version = "0.1.0";
        let hostname = "some-hostname";
        let cgroups_path = "/some/path";

        let spec = SpecBuilder::default()
            .version(version)
            .hostname(hostname)
            .linux(LinuxBuilder::default().cgroups_path(cgroups_path).build()?)
            .build()?;

        assert_eq!(spec.version(), version);
        assert_eq!(
            spec.hostname().as_ref().context("hostname is none")?,
            hostname
        );
        assert_eq!(
            spec.linux()
                .as_ref()
                .context("linux is none")?
                .cgroups_path()
                .as_ref()
                .context("cgroups path is none")?
                .to_str()
                .context("path is not displayable")?,
            cgroups_path
        );

        Ok(())
    }

    #[test]
    fn save_success() -> Result<()> {
        let spec = Spec::default();
        let temp_dir = TempDir::new()?;
        let file = temp_dir.path().join("spec.json");

        spec.save(&file)?;

        let content = fs::read_to_string(&file)?;
        assert!(content.contains("ociVersion"));
        Ok(())
    }

    #[test]
    fn save_fail() -> Result<()> {
        let spec = Spec::default();
        let temp_dir = TempDir::new()?;

        assert!(spec.save(temp_dir.path()).is_err());
        Ok(())
    }

    #[test]
    fn from_file_success() -> Result<()> {
        let temp_file = NamedTempFile::new()?;

        temp_file
            .as_file()
            .write_all(br#"{"ociVersion": "1.0.0"}"#)?;

        let spec = Spec::from(temp_file.path())?;
        assert_eq!(spec.version(), "1.0.0");
        Ok(())
    }

    #[test]
    fn from_file_fail_not_exist() -> Result<()> {
        let path = PathBuf::from("should/not/exist");
        assert!(Spec::from(&path).is_err());
        Ok(())
    }

    #[test]
    fn from_file_fail_deserialize() -> Result<()> {
        let temp_file = NamedTempFile::new()?;

        temp_file.as_file().write_all(b"wrong")?;

        assert!(Spec::from(temp_file.path()).is_err());
        Ok(())
    }
}
