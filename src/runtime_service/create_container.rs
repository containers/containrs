use crate::{
    cri_service::CRIService,
    criapi::{CreateContainerRequest, CreateContainerResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_create_container(
        &self,
        _request: Request<CreateContainerRequest>,
    ) -> Result<Response<CreateContainerResponse>, Status> {
        let resp = CreateContainerResponse {
            container_id: "stub".into(),
        };
        Ok(Response::new(resp))
    }
}
