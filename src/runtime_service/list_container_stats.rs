use crate::{
    cri_service::CRIService,
    criapi::{ListContainerStatsRequest, ListContainerStatsResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_list_container_stats(
        &self,
        _request: Request<ListContainerStatsRequest>,
    ) -> Result<Response<ListContainerStatsResponse>, Status> {
        let resp = ListContainerStatsResponse { stats: vec![] };
        Ok(Response::new(resp))
    }
}
