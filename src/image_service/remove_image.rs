use crate::{
    cri_service::CRIService,
    criapi::{RemoveImageRequest, RemoveImageResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_remove_image(
        &self,
        _request: Request<RemoveImageRequest>,
    ) -> Result<Response<RemoveImageResponse>, Status> {
        let resp = RemoveImageResponse {};
        Ok(Response::new(resp))
    }
}
