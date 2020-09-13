use crate::{
    criapi::{StopPodSandboxRequest, StopPodSandboxResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_stop_pod_sandbox(
        &self,
        _request: Request<StopPodSandboxRequest>,
    ) -> Result<Response<StopPodSandboxResponse>, Status> {
        let reply = StopPodSandboxResponse {};
        Ok(Response::new(reply))
    }
}
