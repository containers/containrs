use crate::kubernetes::cri::{
    api::{StopPodSandboxRequest, StopPodSandboxResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_stop_pod_sandbox stops any running process that is part of the sandbox and reclaims
    /// network resources (e.g., IP addresses) allocated to the sandbox. If there are any running
    /// containers in the sandbox, they must be forcibly terminated. This call is idempotent, and
    /// must not return an error if all relevant resources have already been reclaimed. kubelet
    /// will call StopPodSandbox at least once before calling RemovePodSandbox. It will also
    /// attempt to reclaim resources eagerly, as soon as a sandbox is not needed. Hence, multiple
    /// StopPodSandbox calls are expected.
    pub async fn handle_stop_pod_sandbox(
        &self,
        _request: Request<StopPodSandboxRequest>,
    ) -> Result<Response<StopPodSandboxResponse>, Status> {
        let reply = StopPodSandboxResponse {};
        Ok(Response::new(reply))
    }
}
