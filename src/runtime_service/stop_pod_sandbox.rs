use crate::{
    cri_service::CRIService,
    criapi::{StopPodSandboxRequest, StopPodSandboxResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_stop_pod_sandbox(
        &self,
        _request: Request<StopPodSandboxRequest>,
    ) -> Result<Response<StopPodSandboxResponse>, Status> {
        let reply = StopPodSandboxResponse {};
        Ok(Response::new(reply))
    }
}
