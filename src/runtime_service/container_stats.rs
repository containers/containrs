use crate::{
    criapi::{ContainerStatsRequest, ContainerStatsResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_container_stats(
        &self,
        _request: Request<ContainerStatsRequest>,
    ) -> Result<Response<ContainerStatsResponse>, Status> {
        let resp = ContainerStatsResponse { stats: None };
        Ok(Response::new(resp))
    }
}
