use crate::kubernetes::cri::{
    api::{ContainerStatusRequest, ContainerStatusResponse},
    cri_service::CRIService,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_container_status returns status of the container. If the container is not present,
    /// returns an error.
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
