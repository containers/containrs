use crate::{
    cri_service::CRIService,
    criapi::{PullImageRequest, PullImageResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
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
