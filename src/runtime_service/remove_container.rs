use crate::{
    cri_service::CRIService,
    criapi::{RemoveContainerRequest, RemoveContainerResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_remove_container(
        &self,
        _request: Request<RemoveContainerRequest>,
    ) -> Result<Response<RemoveContainerResponse>, Status> {
        let resp = RemoveContainerResponse {};
        Ok(Response::new(resp))
    }
}
