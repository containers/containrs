//! Kubernetes Container Runtime Interface (CRI) protobuf API
#![allow(missing_docs)]
#![allow(clippy::all)]
use crate::error::ServiceError;
use oci_spec::runtime::MountBuilder;
use std::convert::TryFrom;
use std::fmt::Display;
use std::fs;

use crate::cri::api::Mount as CRIMount;
use oci_spec::runtime::Mount as OCIMount;

include!("runtime.v1alpha2.rs");

impl Display for MountPropagation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let print = match self {
            MountPropagation::PropagationBidirectional => "rshared",
            MountPropagation::PropagationHostToContainer => "rslave",
            MountPropagation::PropagationPrivate => "rprivate",
        };

        write!(f, "{}", print)
    }
}

impl TryFrom<&CRIMount> for OCIMount {
    type Error = ServiceError;

    fn try_from(mount: &CRIMount) -> Result<Self, Self::Error> {
        if mount.container_path.is_empty() {
            return Err(ServiceError::Other(
                "mount container path cannot be empty".to_owned(),
            ));
        }

        if mount.host_path.is_empty() {
            return Err(ServiceError::Other(
                "mount host path cannot be empty".to_owned(),
            ));
        }

        let resolved = fs::read_link(&mount.host_path)?;

        let mut options = Vec::new();
        if mount.readonly {
            options.push("ro".to_owned());
        }

        options.push(mount.propagation().to_string());

        let oci_mount = MountBuilder::default()
            .source(resolved)
            .destination(mount.container_path.as_str())
            .typ("bind")
            .options(options)
            .build()?;

        Ok(oci_mount)
    }
}
