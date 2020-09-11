use crate::criapi::{self, runtime_service_server::RuntimeService};
use tonic::{Request, Response, Status};

mod attach;
mod container_stats;
mod container_status;
mod create_container;
mod exec;
mod exec_sync;
mod list_container_stats;
mod list_containers;
mod list_pod_sandbox;
mod pod_sandbox_status;
mod port_forward;
mod remove_container;
mod remove_pod_sandbox;
mod reopen_container_log;
mod run_pod_sandbox;
mod start_container;
mod status;
mod stop_container;
mod stop_pod_sandbox;
mod update_container_resources;
mod update_runtime_config;
mod version;

#[derive(Default)]
pub struct MyRuntime;

#[tonic::async_trait]
impl RuntimeService for MyRuntime {
    async fn version(
        &self,
        request: Request<criapi::VersionRequest>,
    ) -> Result<Response<criapi::VersionResponse>, Status> {
        self.handle_version(request).await
    }

    async fn create_container(
        &self,
        request: Request<criapi::CreateContainerRequest>,
    ) -> Result<Response<criapi::CreateContainerResponse>, Status> {
        self.handle_create_container(request).await
    }

    async fn start_container(
        &self,
        request: Request<criapi::StartContainerRequest>,
    ) -> Result<Response<criapi::StartContainerResponse>, Status> {
        self.handle_start_container(request).await
    }

    async fn stop_container(
        &self,
        request: Request<criapi::StopContainerRequest>,
    ) -> Result<Response<criapi::StopContainerResponse>, Status> {
        self.handle_stop_container(request).await
    }

    async fn remove_container(
        &self,
        request: Request<criapi::RemoveContainerRequest>,
    ) -> Result<Response<criapi::RemoveContainerResponse>, Status> {
        self.handle_remove_container(request).await
    }

    async fn list_containers(
        &self,
        request: Request<criapi::ListContainersRequest>,
    ) -> Result<Response<criapi::ListContainersResponse>, Status> {
        self.handle_list_containers(request).await
    }

    async fn container_status(
        &self,
        request: Request<criapi::ContainerStatusRequest>,
    ) -> Result<Response<criapi::ContainerStatusResponse>, Status> {
        self.handle_container_status(request).await
    }

    async fn container_stats(
        &self,
        request: Request<criapi::ContainerStatsRequest>,
    ) -> Result<Response<criapi::ContainerStatsResponse>, Status> {
        self.handle_container_stats(request).await
    }

    async fn list_container_stats(
        &self,
        request: Request<criapi::ListContainerStatsRequest>,
    ) -> Result<Response<criapi::ListContainerStatsResponse>, Status> {
        self.handle_list_container_stats(request).await
    }

    async fn update_container_resources(
        &self,
        request: Request<criapi::UpdateContainerResourcesRequest>,
    ) -> Result<Response<criapi::UpdateContainerResourcesResponse>, Status> {
        self.handle_update_container_resources(request).await
    }

    async fn reopen_container_log(
        &self,
        request: Request<criapi::ReopenContainerLogRequest>,
    ) -> Result<Response<criapi::ReopenContainerLogResponse>, Status> {
        self.handle_reopen_container_log(request).await
    }

    async fn exec_sync(
        &self,
        request: Request<criapi::ExecSyncRequest>,
    ) -> Result<Response<criapi::ExecSyncResponse>, Status> {
        self.handle_exec_sync(request).await
    }

    async fn exec(
        &self,
        request: Request<criapi::ExecRequest>,
    ) -> Result<Response<criapi::ExecResponse>, Status> {
        self.handle_exec(request).await
    }

    async fn attach(
        &self,
        request: Request<criapi::AttachRequest>,
    ) -> Result<Response<criapi::AttachResponse>, Status> {
        self.handle_attach(request).await
    }
    async fn port_forward(
        &self,
        request: Request<criapi::PortForwardRequest>,
    ) -> Result<Response<criapi::PortForwardResponse>, Status> {
        self.handle_port_forward(request).await
    }

    async fn run_pod_sandbox(
        &self,
        request: Request<criapi::RunPodSandboxRequest>,
    ) -> Result<Response<criapi::RunPodSandboxResponse>, Status> {
        self.handle_run_pod_sandbox(request).await
    }

    async fn stop_pod_sandbox(
        &self,
        request: Request<criapi::StopPodSandboxRequest>,
    ) -> Result<Response<criapi::StopPodSandboxResponse>, Status> {
        self.handle_stop_pod_sandbox(request).await
    }

    async fn remove_pod_sandbox(
        &self,
        request: Request<criapi::RemovePodSandboxRequest>,
    ) -> Result<Response<criapi::RemovePodSandboxResponse>, Status> {
        self.handle_remove_pod_sandbox(request).await
    }

    async fn list_pod_sandbox(
        &self,
        request: Request<criapi::ListPodSandboxRequest>,
    ) -> Result<Response<criapi::ListPodSandboxResponse>, Status> {
        self.handle_list_pod_sandbox(request).await
    }

    async fn pod_sandbox_status(
        &self,
        request: Request<criapi::PodSandboxStatusRequest>,
    ) -> Result<Response<criapi::PodSandboxStatusResponse>, Status> {
        self.handle_pod_sandbox_status(request).await
    }

    async fn status(
        &self,
        request: Request<criapi::StatusRequest>,
    ) -> Result<Response<criapi::StatusResponse>, Status> {
        self.handle_status(request).await
    }

    async fn update_runtime_config(
        &self,
        request: Request<criapi::UpdateRuntimeConfigRequest>,
    ) -> Result<Response<criapi::UpdateRuntimeConfigResponse>, Status> {
        self.handle_update_runtime_config(request).await
    }
}
