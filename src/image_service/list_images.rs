use crate::{
    criapi::{ListImagesRequest, ListImagesResponse},
    image_service::MyImage,
};
use tonic::{Request, Response, Status};

impl MyImage {
    pub async fn handle_list_images(
        &self,
        _request: Request<ListImagesRequest>,
    ) -> Result<Response<ListImagesResponse>, Status> {
        let resp = ListImagesResponse { images: Vec::new() };
        Ok(Response::new(resp))
    }
}
