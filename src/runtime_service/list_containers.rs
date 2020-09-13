use crate::{
    criapi::{ListContainersRequest, ListContainersResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_list_containers(
        &self,
        _request: Request<ListContainersRequest>,
    ) -> Result<Response<ListContainersResponse>, Status> {
        let resp = ListContainersResponse { containers: vec![] };
        Ok(Response::new(resp))
    }
}
