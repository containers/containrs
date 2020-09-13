use crate::{
    criapi::{RemovePodSandboxRequest, RemovePodSandboxResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_remove_pod_sandbox(
        &self,
        _request: Request<RemovePodSandboxRequest>,
    ) -> Result<Response<RemovePodSandboxResponse>, Status> {
        let reply = RemovePodSandboxResponse {};
        Ok(Response::new(reply))
    }
}
