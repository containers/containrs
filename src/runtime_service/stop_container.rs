use crate::{
    cri_service::CRIService,
    criapi::{StopContainerRequest, StopContainerResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_stop_container(
        &self,
        _request: Request<StopContainerRequest>,
    ) -> Result<Response<StopContainerResponse>, Status> {
        let resp = StopContainerResponse {};
        Ok(Response::new(resp))
    }
}
