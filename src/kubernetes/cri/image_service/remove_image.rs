use crate::kubernetes::cri::{
    api::{RemoveImageRequest, RemoveImageResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_remove_image removes the image. This call is idempotent, and must not return an
    /// error if the image has already been removed.
    pub async fn handle_remove_image(
        &self,
        _request: Request<RemoveImageRequest>,
    ) -> Result<Response<RemoveImageResponse>, Status> {
        let resp = RemoveImageResponse {};
        Ok(Response::new(resp))
    }
}
