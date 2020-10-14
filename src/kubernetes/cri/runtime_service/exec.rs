use crate::kubernetes::cri::{
    api::{ExecRequest, ExecResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_exec prepares a streaming endpoint to execute a command in the container.
    pub async fn handle_exec(
        &self,
        _request: Request<ExecRequest>,
    ) -> Result<Response<ExecResponse>, Status> {
        let resp = ExecResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
