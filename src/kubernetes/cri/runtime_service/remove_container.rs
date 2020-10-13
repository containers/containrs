use crate::kubernetes::cri::{
    api::{RemoveContainerRequest, RemoveContainerResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_remove_container removes the container. If the container is running, the container
    /// must be forcibly removed. This call is idempotent, and must not return an error if the
    /// container has already been removed.
    pub async fn handle_remove_container(
        &self,
        _request: Request<RemoveContainerRequest>,
    ) -> Result<Response<RemoveContainerResponse>, Status> {
        let resp = RemoveContainerResponse {};
        Ok(Response::new(resp))
    }
}
