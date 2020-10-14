use crate::kubernetes::cri::{
    api::{CreateContainerRequest, CreateContainerResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_create_container creates a new container in specified PodSandbox.
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
