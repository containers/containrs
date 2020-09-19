use crate::{
    cri_service::CRIService,
    criapi::{ListContainersRequest, ListContainersResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_list_containers(
        &self,
        _request: Request<ListContainersRequest>,
    ) -> Result<Response<ListContainersResponse>, Status> {
        let resp = ListContainersResponse { containers: vec![] };
        Ok(Response::new(resp))
    }
}
