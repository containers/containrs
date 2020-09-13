use crate::{
    criapi::{ImageFsInfoRequest, ImageFsInfoResponse},
    image_service::MyImage,
};
use tonic::{Request, Response, Status};

impl MyImage {
    pub async fn handle_image_fs_info(
        &self,
        _request: Request<ImageFsInfoRequest>,
    ) -> Result<Response<ImageFsInfoResponse>, Status> {
        let resp = ImageFsInfoResponse {
            image_filesystems: Vec::new(),
        };
        Ok(Response::new(resp))
    }
}
