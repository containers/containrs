use crate::kubernetes::cri::{
    api::{ImageFsInfoRequest, ImageFsInfoResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_image_fs_info returns information of the filesystem that is used to
    /// store images.
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
