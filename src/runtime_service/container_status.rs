use crate::{
    cri_service::CRIService,
    criapi::{ContainerStatusRequest, ContainerStatusResponse},
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
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
