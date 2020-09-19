use crate::{
    cri_service::CRIService,
    criapi::{UpdateContainerResourcesRequest, UpdateContainerResourcesResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_update_container_resources(
        &self,
        _request: Request<UpdateContainerResourcesRequest>,
    ) -> Result<Response<UpdateContainerResourcesResponse>, Status> {
        let resp = UpdateContainerResourcesResponse {};
        Ok(Response::new(resp))
    }
}
