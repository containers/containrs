use crate::{
    criapi::{AttachRequest, AttachResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_attach(
        &self,
        _request: Request<AttachRequest>,
    ) -> Result<Response<AttachResponse>, Status> {
        let resp = AttachResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
