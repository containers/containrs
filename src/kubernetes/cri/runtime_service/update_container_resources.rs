use crate::kubernetes::cri::{
    api::{UpdateContainerResourcesRequest, UpdateContainerResourcesResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_update_container_resources updates ContainerConfig of the container.
    pub async fn handle_update_container_resources(
        &self,
        _request: Request<UpdateContainerResourcesRequest>,
    ) -> Result<Response<UpdateContainerResourcesResponse>, Status> {
        let resp = UpdateContainerResourcesResponse {};
        Ok(Response::new(resp))
    }
}
