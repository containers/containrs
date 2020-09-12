use crate::{
    cri_service::CRIService,
    criapi::{ListPodSandboxRequest, ListPodSandboxResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_list_pod_sandbox(
        &self,
        _request: Request<ListPodSandboxRequest>,
    ) -> Result<Response<ListPodSandboxResponse>, Status> {
        let reply = ListPodSandboxResponse { items: vec![] };
        Ok(Response::new(reply))
    }
}
