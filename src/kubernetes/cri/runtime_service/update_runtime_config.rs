use crate::kubernetes::cri::{
    api::{UpdateRuntimeConfigRequest, UpdateRuntimeConfigResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_update_runtime_config updates the runtime configuration based on the given request.
    pub async fn handle_update_runtime_config(
        &self,
        _request: Request<UpdateRuntimeConfigRequest>,
    ) -> Result<Response<UpdateRuntimeConfigResponse>, Status> {
        let resp = UpdateRuntimeConfigResponse {};
        Ok(Response::new(resp))
    }
}
