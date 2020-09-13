use crate::{
    criapi::{UpdateRuntimeConfigRequest, UpdateRuntimeConfigResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_update_runtime_config(
        &self,
        _request: Request<UpdateRuntimeConfigRequest>,
    ) -> Result<Response<UpdateRuntimeConfigResponse>, Status> {
        let resp = UpdateRuntimeConfigResponse {};
        Ok(Response::new(resp))
    }
}
