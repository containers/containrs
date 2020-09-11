use crate::{
    criapi::{StartContainerRequest, StartContainerResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_start_container(
        &self,
        _request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        let resp = StartContainerResponse {};
        Ok(Response::new(resp))
    }
}
