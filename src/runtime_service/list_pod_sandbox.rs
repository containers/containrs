use crate::{
    criapi::{ListPodSandboxRequest, ListPodSandboxResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_list_pod_sandbox(
        &self,
        _request: Request<ListPodSandboxRequest>,
    ) -> Result<Response<ListPodSandboxResponse>, Status> {
        let reply = ListPodSandboxResponse { items: vec![] };
        Ok(Response::new(reply))
    }
}
