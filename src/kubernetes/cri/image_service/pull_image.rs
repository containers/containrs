use crate::kubernetes::cri::{
    api::{PullImageRequest, PullImageResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_pull_image pulls an image with authentication config.
    pub async fn handle_pull_image(
        &self,
        _request: Request<PullImageRequest>,
    ) -> Result<Response<PullImageResponse>, Status> {
        let resp = PullImageResponse {
            image_ref: "some_image".into(),
        };
        Ok(Response::new(resp))
    }
}
