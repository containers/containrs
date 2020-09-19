use crate::{
    cri_service::CRIService,
    criapi::{RemovePodSandboxRequest, RemovePodSandboxResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_remove_pod_sandbox(
        &self,
        _request: Request<RemovePodSandboxRequest>,
    ) -> Result<Response<RemovePodSandboxResponse>, Status> {
        let reply = RemovePodSandboxResponse {};
        Ok(Response::new(reply))
    }
}
