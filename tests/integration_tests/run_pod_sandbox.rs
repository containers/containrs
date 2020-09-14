use crate::common::{
    criapi::{PodSandboxConfig, PodSandboxMetadata, RunPodSandboxRequest, RunPodSandboxResponse},
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
            linux: None,
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
