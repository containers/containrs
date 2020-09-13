use crate::{
    criapi::{ExecRequest, ExecResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_exec(
        &self,
        _request: Request<ExecRequest>,
    ) -> Result<Response<ExecResponse>, Status> {
        let resp = ExecResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
