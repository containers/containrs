use crate::{
    cri_service::CRIService,
    criapi::{ListImagesRequest, ListImagesResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_list_images(
        &self,
        _request: Request<ListImagesRequest>,
    ) -> Result<Response<ListImagesResponse>, Status> {
        let resp = ListImagesResponse { images: Vec::new() };
        Ok(Response::new(resp))
    }
}
