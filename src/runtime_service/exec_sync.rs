use crate::{
    cri_service::CRIService,
    criapi::{ExecSyncRequest, ExecSyncResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
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
