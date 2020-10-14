use crate::kubernetes::cri::{
    api::{ListPodSandboxRequest, ListPodSandboxResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_list_pod_sandbox returns a list of PodSandboxes.
    pub async fn handle_list_pod_sandbox(
        &self,
        _request: Request<ListPodSandboxRequest>,
    ) -> Result<Response<ListPodSandboxResponse>, Status> {
        let reply = ListPodSandboxResponse { items: vec![] };
        Ok(Response::new(reply))
    }
}
