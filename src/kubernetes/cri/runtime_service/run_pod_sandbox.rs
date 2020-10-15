use crate::{
    kubernetes::cri::{
        api::{NamespaceMode, RunPodSandboxRequest, RunPodSandboxResponse},
        cri_service::{CRIService, OptionStatus, ResultStatus},
    },
    sandbox::{pinned::PinnedSandbox, LinuxNamespaces, SandboxBuilder, SandboxDataBuilder},
};
use log::{debug, info};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_run_pod_sandbox creates and starts a pod-level sandbox. Runtimes must ensure the
    /// sandbox is in the ready state on success.
    pub async fn handle_run_pod_sandbox(
        &self,
        request: Request<RunPodSandboxRequest>,
    ) -> Result<Response<RunPodSandboxResponse>, Status> {
        // Take the pod sandbox config
        let config = request
            .into_inner()
            .config
            .take()
            .ok_or_invalid("no pod sandbox config provided")?;

        // Verify that the metadata exists
        let metadata = config
            .metadata
            .ok_or_invalid("no pod sandbox metadata provided")?;

        let linux_config = config
            .linux
            .ok_or_invalid("no linux configuration provided")?;

        let security_context = linux_config
            .security_context
            .ok_or_invalid("no linux security context provided")?;

        let namespace_options = security_context
            .namespace_options
            .ok_or_invalid("no namespace options provided")?;

        let mut linux_namespaces = LinuxNamespaces::empty();
        if namespace_options.network == NamespaceMode::Pod as i32 {
            linux_namespaces |= LinuxNamespaces::NET;
            linux_namespaces |= LinuxNamespaces::UTS;
        }
        if namespace_options.ipc == NamespaceMode::Pod as i32 {
            linux_namespaces |= LinuxNamespaces::IPC;
        }
        if namespace_options.pid == NamespaceMode::Pod as i32 {
            linux_namespaces |= LinuxNamespaces::PID;
        }

        // Build a new sandbox from it
        let mut sandbox = SandboxBuilder::<PinnedSandbox>::default()
            .data(
                SandboxDataBuilder::default()
                    .id(metadata.uid)
                    .name(metadata.name)
                    .namespace(metadata.namespace)
                    .attempt(metadata.attempt)
                    .linux_namespaces(linux_namespaces)
                    .hostname(config.hostname)
                    .log_directory(config.log_directory)
                    .annotations(config.annotations)
                    .build()
                    .map_internal("build sandbox data from metadata")?,
            )
            .build()
            .map_internal("build sandbox from config")?;

        debug!("Created pod sandbox {:?}", sandbox);

        // Run the sandbox
        sandbox.run().map_internal("run pod sandbox")?;
        info!("Started pod sandbox {}", sandbox);

        // Build and return the response
        let reply = RunPodSandboxResponse {
            pod_sandbox_id: sandbox.id().into(),
        };
        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::cri::{
        api::{
            runtime_service_server::RuntimeService, LinuxPodSandboxConfig,
            LinuxSandboxSecurityContext, NamespaceOption, PodSandboxConfig, PodSandboxMetadata,
        },
        cri_service::tests::new_cri_service,
    };
    use anyhow::Result;
    use std::collections::HashMap;

    #[tokio::test]
    async fn run_pod_sandbox_success() -> Result<()> {
        let sut = new_cri_service()?;
        let test_id = "123";
        let request = RunPodSandboxRequest {
            config: Some(PodSandboxConfig {
                metadata: Some(PodSandboxMetadata {
                    name: "".into(),
                    uid: test_id.into(),
                    namespace: "".into(),
                    attempt: 0,
                }),
                hostname: "".into(),
                log_directory: "".into(),
                dns_config: None,
                port_mappings: vec![],
                labels: HashMap::new(),
                annotations: HashMap::new(),
                linux: Some(LinuxPodSandboxConfig {
                    cgroup_parent: String::from("abc-pod.slice"),
                    sysctls: HashMap::new(),
                    security_context: Some(LinuxSandboxSecurityContext {
                        namespace_options: Some(NamespaceOption {
                            network: 0,
                            pid: 1,
                            ipc: 0,
                            target_id: String::from("container_id"),
                        }),
                        selinux_options: None,
                        run_as_user: None,
                        run_as_group: None,
                        readonly_rootfs: false,
                        supplemental_groups: Vec::new(),
                        privileged: false,
                        seccomp_profile_path: String::from("/path/to/seccomp"),
                    }),
                }),
            }),
            runtime_handler: "".into(),
        };
        let response = sut.run_pod_sandbox(Request::new(request)).await?;
        assert_eq!(response.get_ref().pod_sandbox_id, test_id);
        Ok(())
    }

    #[tokio::test]
    async fn run_pod_sandbox_fail_no_config() -> Result<()> {
        let sut = new_cri_service()?;
        let request = RunPodSandboxRequest {
            config: None,
            runtime_handler: "".into(),
        };
        let response = sut.run_pod_sandbox(Request::new(request)).await;
        assert!(response.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn run_pod_sandbox_fail_no_config_metadata() -> Result<()> {
        let sut = new_cri_service()?;
        let request = RunPodSandboxRequest {
            config: Some(PodSandboxConfig {
                metadata: None,
                hostname: "".into(),
                log_directory: "".into(),
                dns_config: None,
                port_mappings: vec![],
                labels: HashMap::new(),
                annotations: HashMap::new(),
                linux: None,
            }),
            runtime_handler: "".into(),
        };
        let response = sut.run_pod_sandbox(Request::new(request)).await;
        assert!(response.is_err());
        Ok(())
    }
}
