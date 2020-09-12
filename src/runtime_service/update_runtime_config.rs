use crate::{
    cri_service::CRIService,
    criapi::{UpdateRuntimeConfigRequest, UpdateRuntimeConfigResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_update_runtime_config(
        &self,
        _request: Request<UpdateRuntimeConfigRequest>,
    ) -> Result<Response<UpdateRuntimeConfigResponse>, Status> {
        let resp = UpdateRuntimeConfigResponse {};
        Ok(Response::new(resp))
    }
}
