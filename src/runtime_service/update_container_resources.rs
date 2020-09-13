use crate::{
    criapi::{UpdateContainerResourcesRequest, UpdateContainerResourcesResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_update_container_resources(
        &self,
        _request: Request<UpdateContainerResourcesRequest>,
    ) -> Result<Response<UpdateContainerResourcesResponse>, Status> {
        let resp = UpdateContainerResourcesResponse {};
        Ok(Response::new(resp))
    }
}
