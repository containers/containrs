use crate::kubernetes::cri::{
    api::{ReopenContainerLogRequest, ReopenContainerLogResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_reopen_container_log asks runtime to reopen the stdout/stderr log file for the
    /// container. This is often called after the log file has been rotated. If the container is
    /// not running, container runtime can choose to either create a new log file and return None,
    /// or return an error. Once it returns error, new container log file MUST NOT be created.
    pub async fn handle_reopen_container_log(
        &self,
        _request: Request<ReopenContainerLogRequest>,
    ) -> Result<Response<ReopenContainerLogResponse>, Status> {
        let resp = ReopenContainerLogResponse {};
        Ok(Response::new(resp))
    }
}
