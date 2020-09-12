use crate::{
    cri_service::CRIService,
    criapi::{StatusRequest, StatusResponse},
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let resp = StatusResponse {
            status: None,
            info: HashMap::new(),
        };
        Ok(Response::new(resp))
    }
}
