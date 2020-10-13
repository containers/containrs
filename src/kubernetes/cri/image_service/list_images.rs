use crate::kubernetes::cri::{
    api::{ListImagesRequest, ListImagesResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_list_images lists existing images.
    pub async fn handle_list_images(
        &self,
        _request: Request<ListImagesRequest>,
    ) -> Result<Response<ListImagesResponse>, Status> {
        let resp = ListImagesResponse { images: Vec::new() };
        Ok(Response::new(resp))
    }
}
