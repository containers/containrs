use crate::{
    cri_service::CRIService,
    criapi::{PodSandboxStatusRequest, PodSandboxStatusResponse},
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
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
