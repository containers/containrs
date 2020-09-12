use crate::{
    cri_service::CRIService,
    criapi::{ExecRequest, ExecResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_exec(
        &self,
        _request: Request<ExecRequest>,
    ) -> Result<Response<ExecResponse>, Status> {
        let resp = ExecResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
