use crate::{
    criapi::image_service_server::ImageServiceServer,
    criapi::runtime_service_server::RuntimeServiceServer,
    runtime::{MyImage, MyRuntime},
};
use tonic::transport;

/// Server is the main instance to run the Container Runtime Interface
pub struct Server;

impl Server {
    /// Start a new server with its default values
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        let addr = "[::1]:50051".parse().unwrap();
        let rt = MyRuntime::default();
        let img = MyImage::default();

        println!("Runtime server listening on {}", addr);

        transport::Server::builder()
            .add_service(RuntimeServiceServer::new(rt))
            .add_service(ImageServiceServer::new(img))
            .serve(addr)
            .await?;

        Ok(())
    }
}
