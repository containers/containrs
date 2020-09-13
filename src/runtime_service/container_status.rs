use crate::{
    criapi::{ContainerStatusRequest, ContainerStatusResponse},
    runtime_service::MyRuntime,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_container_status(
        &self,
        _request: Request<ContainerStatusRequest>,
    ) -> Result<Response<ContainerStatusResponse>, Status> {
        let resp = ContainerStatusResponse {
            info: HashMap::new(),
            status: None,
        };
        Ok(Response::new(resp))
    }
}
