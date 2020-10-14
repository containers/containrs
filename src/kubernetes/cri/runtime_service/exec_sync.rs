use crate::kubernetes::cri::{
    api::{ExecSyncRequest, ExecSyncResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_exec_sync runs a command in a container synchronously.
    pub async fn handle_exec_sync(
        &self,
        _request: Request<ExecSyncRequest>,
    ) -> Result<Response<ExecSyncResponse>, Status> {
        let resp = ExecSyncResponse {
            exit_code: -1,
            stderr: Vec::new(),
            stdout: Vec::new(),
        };
        Ok(Response::new(resp))
    }
}
