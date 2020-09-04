use tonic::{transport::Server, Request, Response, Status};
use std::vec::Vec;

pub mod criapi {
    tonic::include_proto!("criapi");
}
use criapi::runtime_service_server::{RuntimeService,RuntimeServiceServer};

#[derive(Default)]
pub struct MyRuntime {}

#[tonic::async_trait]
impl RuntimeService for MyRuntime {
    async fn version(
        &self,
        request: Request<criapi::VersionRequest>,
    ) -> Result<Response<criapi::VersionResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let resp = criapi::VersionResponse {
            version: String::from("0.1.0"),
            runtime_api_version: String::from("v1alpha1"),
            runtime_name: String::from("crust"),
            runtime_version: String::from("0.0.1"),
        };
        Ok(Response::new(resp))
    }

    async fn create_container(
        &self,
        request: Request<criapi::CreateContainerRequest>,
    ) -> Result<Response<criapi::CreateContainerResponse>, Status> {
        let resp = criapi::CreateContainerResponse {
            container_id: String::from("stub"),
        };
        Ok(Response::new(resp))
    }

    async fn start_container(
        &self,
        request: Request<criapi::StartContainerRequest>,
    ) -> Result<Response<criapi::StartContainerResponse>, Status> {
        let resp = criapi::StartContainerResponse {
        };
        Ok(Response::new(resp))
    }

    async fn stop_container(
        &self,
        request: Request<criapi::StopContainerRequest>,
    ) -> Result<Response<criapi::StopContainerResponse>, Status> {
        let resp = criapi::StopContainerResponse {
        };
        Ok(Response::new(resp))
    }

    async fn remove_container(
        &self,
        request: Request<criapi::RemoveContainerRequest>,
    ) -> Result<Response<criapi::RemoveContainerResponse>, Status> {
        let resp = criapi::RemoveContainerResponse {
        };
        Ok(Response::new(resp))
    }

    async fn list_containers(
        &self,
        request: Request<criapi::ListContainersRequest>,
    ) -> Result<Response<criapi::ListContainersResponse>, Status> {
        let resp = criapi::ListContainersResponse {
            containers: Vec::new()
        };
        Ok(Response::new(resp))
    }
    async fn run_pod_sandbox(
        &self,
        request: Request<criapi::RunPodSandboxRequest>,
    ) -> Result<Response<criapi::RunPodSandboxResponse>, Status> {
        let reply = criapi::RunPodSandboxResponse {
            pod_sandbox_id: String::from("1234567890"),
        };
        Ok(Response::new(reply))
    }
    async fn stop_pod_sandbox(
        &self,
        request: Request<criapi::StopPodSandboxRequest>,
    ) -> Result<Response<criapi::StopPodSandboxResponse>, Status> {
        let reply = criapi::StopPodSandboxResponse {};
        Ok(Response::new(reply))
    }
    async fn remove_pod_sandbox(
        &self,
        request: Request<criapi::RemovePodSandboxRequest>,
    ) -> Result<Response<criapi::RemovePodSandboxResponse>, Status> {
        let reply = criapi::RemovePodSandboxResponse {};
        Ok(Response::new(reply))
    }
    async fn list_pod_sandbox(
        &self,
        request: Request<criapi::ListPodSandboxRequest>,
    ) -> Result<Response<criapi::ListPodSandboxResponse>, Status> {
        let reply = criapi::ListPodSandboxResponse {
            items: Vec::new(),
        };
        Ok(Response::new(reply))
    }
    async fn pod_sandbox_status(
        &self,
        request: Request<criapi::PodSandboxStatusRequest>,
    ) -> Result<Response<criapi::PodSandboxStatusResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let rt = MyRuntime::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(RuntimeServiceServer::new(rt))
        .serve(addr)
        .await?;

    Ok(())
}
