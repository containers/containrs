use crate::{
    criapi::{StopContainerRequest, StopContainerResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_stop_container(
        &self,
        _request: Request<StopContainerRequest>,
    ) -> Result<Response<StopContainerResponse>, Status> {
        let resp = StopContainerResponse {};
        Ok(Response::new(resp))
    }
}
