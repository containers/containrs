use crate::kubernetes::cri::{
    api::{ImageStatusRequest, ImageStatusResponse},
    cri_service::CRIService,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_image_status returns the status of the image. If the image is not
    /// present, returns a response with ImageStatusResponse.image set to
    /// None.
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
