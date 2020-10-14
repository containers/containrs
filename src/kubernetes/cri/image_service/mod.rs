use crate::kubernetes::cri::{
    api::{self, image_service_server::ImageService},
    cri_service::CRIService,
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
        request: Request<api::ListImagesRequest>,
    ) -> Result<Response<api::ListImagesResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_images(request).await;
        self.debug_response(&response);
        response
    }

    async fn pull_image(
        &self,
        request: Request<api::PullImageRequest>,
    ) -> Result<Response<api::PullImageResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_pull_image(request).await;
        self.debug_response(&response);
        response
    }

    async fn image_status(
        &self,
        request: Request<api::ImageStatusRequest>,
    ) -> Result<Response<api::ImageStatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_image_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn remove_image(
        &self,
        request: Request<api::RemoveImageRequest>,
    ) -> Result<Response<api::RemoveImageResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_remove_image(request).await;
        self.debug_response(&response);
        response
    }

    async fn image_fs_info(
        &self,
        request: Request<api::ImageFsInfoRequest>,
    ) -> Result<Response<api::ImageFsInfoResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_image_fs_info(request).await;
        self.debug_response(&response);
        response
    }
}
