use crate::kubernetes::cri::{
    api::{ListContainerStatsRequest, ListContainerStatsResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_list_container_stats returns stats of all running containers.
    pub async fn handle_list_container_stats(
        &self,
        _request: Request<ListContainerStatsRequest>,
    ) -> Result<Response<ListContainerStatsResponse>, Status> {
        let resp = ListContainerStatsResponse { stats: vec![] };
        Ok(Response::new(resp))
    }
}
