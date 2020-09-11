use crate::{
    criapi::{CreateContainerRequest, CreateContainerResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
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
