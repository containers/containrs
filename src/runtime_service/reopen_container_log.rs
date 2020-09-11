use crate::{
    criapi::{ReopenContainerLogRequest, ReopenContainerLogResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_reopen_container_log(
        &self,
        _request: Request<ReopenContainerLogRequest>,
    ) -> Result<Response<ReopenContainerLogResponse>, Status> {
        let resp = ReopenContainerLogResponse {};
        Ok(Response::new(resp))
    }
}
