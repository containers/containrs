use crate::{
    criapi::{ListContainerStatsRequest, ListContainerStatsResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_list_container_stats(
        &self,
        _request: Request<ListContainerStatsRequest>,
    ) -> Result<Response<ListContainerStatsResponse>, Status> {
        let resp = ListContainerStatsResponse { stats: vec![] };
        Ok(Response::new(resp))
    }
}
