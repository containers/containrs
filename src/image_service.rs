use crate::criapi::{self, image_service_server::ImageService};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MyImage {}

#[tonic::async_trait]
impl ImageService for MyImage {
    async fn list_images(
        &self,
        _request: Request<criapi::ListImagesRequest>,
    ) -> Result<Response<criapi::ListImagesResponse>, Status> {
        let resp = criapi::ListImagesResponse { images: Vec::new() };
        Ok(Response::new(resp))
    }

    async fn pull_image(
        &self,
        _request: Request<criapi::PullImageRequest>,
    ) -> Result<Response<criapi::PullImageResponse>, Status> {
        let resp = criapi::PullImageResponse {
            image_ref: "some_image".into(),
        };
        Ok(Response::new(resp))
    }

    async fn image_status(
        &self,
        _request: Request<criapi::ImageStatusRequest>,
    ) -> Result<Response<criapi::ImageStatusResponse>, Status> {
        let resp = criapi::ImageStatusResponse {
            image: None,
            info: HashMap::new(),
        };
        Ok(Response::new(resp))
    }

    async fn remove_image(
        &self,
        _request: Request<criapi::RemoveImageRequest>,
    ) -> Result<Response<criapi::RemoveImageResponse>, Status> {
        let resp = criapi::RemoveImageResponse {};
        Ok(Response::new(resp))
    }

    async fn image_fs_info(
        &self,
        _request: Request<criapi::ImageFsInfoRequest>,
    ) -> Result<Response<criapi::ImageFsInfoResponse>, Status> {
        let resp = criapi::ImageFsInfoResponse {
            image_filesystems: Vec::new(),
        };
        Ok(Response::new(resp))
    }
}
