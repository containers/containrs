//! A pod sandbox implementation which does pin it's namespaces to file descriptors.

use crate::sandbox::Pod;

#[derive(Default)]
pub struct PinnedSandbox {}

impl Pod for PinnedSandbox {}
