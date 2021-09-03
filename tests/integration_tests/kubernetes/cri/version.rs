use crate::common::{
    criapi::{VersionRequest, VersionResponse},
    Sut,
};
use anyhow::Result;
use tonic::Request;

#[tokio::test]
async fn version_success() -> Result<()> {
    // Given
    let mut sut = Sut::start().await?;
    let request = Request::new(VersionRequest {
        version: "0.1.0".into(),
    });

    // When
    let response = sut
        .runtime_client_mut()
        .version(request)
        .await?
        .into_inner();

    // Then
    assert_eq!(
        response,
        VersionResponse {
            version: "0.1.0".into(),
            runtime_api_version: "v1alpha1".into(),
            runtime_name: "crust".into(),
            runtime_version: "0.0.1".into(),
        }
    );

    sut.cleanup()
}
