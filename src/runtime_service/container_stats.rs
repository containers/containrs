use crate::{
    cri_service::CRIService,
    criapi::{ContainerStatsRequest, ContainerStatsResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_container_stats(
        &self,
        _request: Request<ContainerStatsRequest>,
    ) -> Result<Response<ContainerStatsResponse>, Status> {
        let resp = ContainerStatsResponse { stats: None };
        Ok(Response::new(resp))
    }
}
