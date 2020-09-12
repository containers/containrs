use crate::{
    cri_service::CRIService,
    criapi::{StartContainerRequest, StartContainerResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_start_container(
        &self,
        _request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        let resp = StartContainerResponse {};
        Ok(Response::new(resp))
    }
}
