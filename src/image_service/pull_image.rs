use crate::{
    criapi::{PullImageRequest, PullImageResponse},
    image_service::MyImage,
};
use tonic::{Request, Response, Status};

impl MyImage {
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
