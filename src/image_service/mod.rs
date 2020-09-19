use crate::{
    cri_service::CRIService,
    criapi::{self, image_service_server::ImageService},
};
use tonic::{Request, Response, Status};

mod image_fs_info;
mod image_status;
mod list_images;
mod pull_image;
mod remove_image;

#[tonic::async_trait]
impl ImageService for CRIService {
    async fn list_images(
        &self,
        request: Request<criapi::ListImagesRequest>,
    ) -> Result<Response<criapi::ListImagesResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_images(request).await;
        self.debug_response(&response);
        response
    }

    async fn pull_image(
        &self,
        request: Request<criapi::PullImageRequest>,
    ) -> Result<Response<criapi::PullImageResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_pull_image(request).await;
        self.debug_response(&response);
        response
    }

    async fn image_status(
        &self,
        request: Request<criapi::ImageStatusRequest>,
    ) -> Result<Response<criapi::ImageStatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_image_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn remove_image(
        &self,
        request: Request<criapi::RemoveImageRequest>,
    ) -> Result<Response<criapi::RemoveImageResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_remove_image(request).await;
        self.debug_response(&response);
        response
    }

    async fn image_fs_info(
        &self,
        request: Request<criapi::ImageFsInfoRequest>,
    ) -> Result<Response<criapi::ImageFsInfoResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_image_fs_info(request).await;
        self.debug_response(&response);
        response
    }
}
