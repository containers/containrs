use crate::{
    criapi::{PodSandboxStatusRequest, PodSandboxStatusResponse},
    runtime_service::MyRuntime,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl MyRuntime {
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
