use crate::kubernetes::cri::{
    api::{StopContainerRequest, StopContainerResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_stop_container stops a running container with a grace period (i.e., timeout). This
    /// call is idempotent, and must not return an error if the container has already been stopped.
    pub async fn handle_stop_container(
        &self,
        _request: Request<StopContainerRequest>,
    ) -> Result<Response<StopContainerResponse>, Status> {
        let resp = StopContainerResponse {};
        Ok(Response::new(resp))
    }
}
