use crate::{
    cri_service::CRIService,
    criapi::{ImageFsInfoRequest, ImageFsInfoResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
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
