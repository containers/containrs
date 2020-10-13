use crate::kubernetes::cri::{
    api::{VersionRequest, VersionResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_version returns the runtime name, runtime version, and runtime API version.
    pub async fn handle_version(
        &self,
        _request: Request<VersionRequest>,
    ) -> Result<Response<VersionResponse>, Status> {
        let resp = VersionResponse {
            version: "0.1.0".into(),
            runtime_api_version: "v1alpha1".into(),
            runtime_name: "crust".into(),
            runtime_version: "0.0.1".into(),
        };
        Ok(Response::new(resp))
    }
}
