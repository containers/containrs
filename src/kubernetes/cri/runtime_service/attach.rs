use crate::kubernetes::cri::{
    api::{AttachRequest, AttachResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_attach prepares a streaming endpoint to attach to a running container.
    pub async fn handle_attach(
        &self,
        _request: Request<AttachRequest>,
    ) -> Result<Response<AttachResponse>, Status> {
        let resp = AttachResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
