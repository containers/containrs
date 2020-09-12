use crate::{
    cri_service::CRIService,
    criapi::{AttachRequest, AttachResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_attach(
        &self,
        _request: Request<AttachRequest>,
    ) -> Result<Response<AttachResponse>, Status> {
        let resp = AttachResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
