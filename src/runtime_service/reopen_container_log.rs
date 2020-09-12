use crate::{
    cri_service::CRIService,
    criapi::{ReopenContainerLogRequest, ReopenContainerLogResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_reopen_container_log(
        &self,
        _request: Request<ReopenContainerLogRequest>,
    ) -> Result<Response<ReopenContainerLogResponse>, Status> {
        let resp = ReopenContainerLogResponse {};
        Ok(Response::new(resp))
    }
}
