use crate::kubernetes::cri::{
    api::{PodSandboxStatusRequest, PodSandboxStatusResponse},
    cri_service::CRIService,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_pod_sandbox_status returns the status of the PodSandbox. If the PodSandbox is not
    /// present, returns an error.
    pub async fn handle_pod_sandbox_status(
        &self,
        _request: Request<PodSandboxStatusRequest>,
    ) -> Result<Response<PodSandboxStatusResponse>, Status> {
        let reply = PodSandboxStatusResponse {
            info: HashMap::new(),
            status: None,
        };
        Ok(Response::new(reply))
    }
}
