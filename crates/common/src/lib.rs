use std::path::PathBuf;

pub mod capability;
pub mod seccomp;
pub mod unix_stream;

#[derive(Clone, Debug)]
pub struct Namespace {
    pub typ: NamespaceType,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub enum NamespaceType {
    UTS,
    IPC,
    USER,
    NET,
    MOUNT,
    PID,
}
