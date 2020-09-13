use crate::{
    criapi::{RemoveContainerRequest, RemoveContainerResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_remove_container(
        &self,
        _request: Request<RemoveContainerRequest>,
    ) -> Result<Response<RemoveContainerResponse>, Status> {
        let resp = RemoveContainerResponse {};
        Ok(Response::new(resp))
    }
}
