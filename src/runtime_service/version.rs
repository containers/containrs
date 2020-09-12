use crate::{
    cri_service::CRIService,
    criapi::{VersionRequest, VersionResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
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
