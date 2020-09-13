use crate::{
    criapi::{StatusRequest, StatusResponse},
    runtime_service::MyRuntime,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl MyRuntime {
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
