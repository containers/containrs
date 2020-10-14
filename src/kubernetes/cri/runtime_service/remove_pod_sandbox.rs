use crate::kubernetes::cri::{
    api::{RemovePodSandboxRequest, RemovePodSandboxResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_remove_pod_sandbox removes the sandbox. If there are any running containers in the
    /// sandbox, they must be forcibly terminated and removed.  This call is idempotent, and must
    /// not return an error if the sandbox has already been removed.
    pub async fn handle_remove_pod_sandbox(
        &self,
        _request: Request<RemovePodSandboxRequest>,
    ) -> Result<Response<RemovePodSandboxResponse>, Status> {
        let reply = RemovePodSandboxResponse {};
        Ok(Response::new(reply))
    }
}
