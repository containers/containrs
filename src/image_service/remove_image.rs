use crate::{
    criapi::{RemoveImageRequest, RemoveImageResponse},
    image_service::MyImage,
};
use tonic::{Request, Response, Status};

impl MyImage {
    pub async fn handle_remove_image(
        &self,
        _request: Request<RemoveImageRequest>,
    ) -> Result<Response<RemoveImageResponse>, Status> {
        let resp = RemoveImageResponse {};
        Ok(Response::new(resp))
    }
}
