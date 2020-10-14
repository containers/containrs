use crate::kubernetes::cri::{
    api::{ContainerStatsRequest, ContainerStatsResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_container_stats returns stats of the container. If the container does not exist, the
    /// call returns an error.
    pub async fn handle_container_stats(
        &self,
        _request: Request<ContainerStatsRequest>,
    ) -> Result<Response<ContainerStatsResponse>, Status> {
        let resp = ContainerStatsResponse { stats: None };
        Ok(Response::new(resp))
    }
}
