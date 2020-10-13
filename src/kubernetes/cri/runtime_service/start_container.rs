use crate::kubernetes::cri::{
    api::{StartContainerRequest, StartContainerResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_start_container starts the container.
    pub async fn handle_start_container(
        &self,
        _request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        let resp = StartContainerResponse {};
        Ok(Response::new(resp))
    }
}
