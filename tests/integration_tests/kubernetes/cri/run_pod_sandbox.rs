use crate::common::{
    criapi::{
        LinuxPodSandboxConfig, LinuxSandboxSecurityContext, NamespaceOption, PodSandboxConfig,
        PodSandboxMetadata, RunPodSandboxRequest, RunPodSandboxResponse,
    },
    Sut,
};
use anyhow::Result;
use std::collections::HashMap;
use tonic::Request;

#[tokio::test]
async fn run_pod_sandbox_success() -> Result<()> {
    // Given
    let mut sut = Sut::start().await?;
    let test_id = "123";
    let request = Request::new(RunPodSandboxRequest {
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
    });

    // When
    let response = sut
        .runtime_client_mut()
        .run_pod_sandbox(request)
        .await?
        .into_inner();

    // Then
    assert_eq!(
        response,
        RunPodSandboxResponse {
            pod_sandbox_id: test_id.into()
        }
    );

    sut.cleanup()
}
