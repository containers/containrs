use crate::kubernetes::cri::{
    api::{ListContainersRequest, ListContainersResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_list_containers lists all containers by filters.
    pub async fn handle_list_containers(
        &self,
        _request: Request<ListContainersRequest>,
    ) -> Result<Response<ListContainersResponse>, Status> {
        let resp = ListContainersResponse { containers: vec![] };
        Ok(Response::new(resp))
    }
}
