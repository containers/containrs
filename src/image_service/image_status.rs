use crate::{
    criapi::{ImageStatusRequest, ImageStatusResponse},
    image_service::MyImage,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl MyImage {
    pub async fn handle_image_status(
        &self,
        _request: Request<ImageStatusRequest>,
    ) -> Result<Response<ImageStatusResponse>, Status> {
        let resp = ImageStatusResponse {
            image: None,
            info: HashMap::new(),
        };
        Ok(Response::new(resp))
    }
}
