use crate::{
    cri_service::CRIService,
    criapi::{self, runtime_service_server::RuntimeService},
};
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

#[tonic::async_trait]
impl RuntimeService for CRIService {
    async fn version(
        &self,
        request: Request<criapi::VersionRequest>,
    ) -> Result<Response<criapi::VersionResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_version(request).await;
        self.debug_response(&response);
        response
    }

    async fn create_container(
        &self,
        request: Request<criapi::CreateContainerRequest>,
    ) -> Result<Response<criapi::CreateContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_create_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn start_container(
        &self,
        request: Request<criapi::StartContainerRequest>,
    ) -> Result<Response<criapi::StartContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_start_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn stop_container(
        &self,
        request: Request<criapi::StopContainerRequest>,
    ) -> Result<Response<criapi::StopContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_stop_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn remove_container(
        &self,
        request: Request<criapi::RemoveContainerRequest>,
    ) -> Result<Response<criapi::RemoveContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_remove_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn list_containers(
        &self,
        request: Request<criapi::ListContainersRequest>,
    ) -> Result<Response<criapi::ListContainersResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_containers(request).await;
        self.debug_response(&response);
        response
    }

    async fn container_status(
        &self,
        request: Request<criapi::ContainerStatusRequest>,
    ) -> Result<Response<criapi::ContainerStatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_container_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn container_stats(
        &self,
        request: Request<criapi::ContainerStatsRequest>,
    ) -> Result<Response<criapi::ContainerStatsResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_container_stats(request).await;
        self.debug_response(&response);
        response
    }

    async fn list_container_stats(
        &self,
        request: Request<criapi::ListContainerStatsRequest>,
    ) -> Result<Response<criapi::ListContainerStatsResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_container_stats(request).await;
        self.debug_response(&response);
        response
    }

    async fn update_container_resources(
        &self,
        request: Request<criapi::UpdateContainerResourcesRequest>,
    ) -> Result<Response<criapi::UpdateContainerResourcesResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_update_container_resources(request).await;
        self.debug_response(&response);
        response
    }

    async fn reopen_container_log(
        &self,
        request: Request<criapi::ReopenContainerLogRequest>,
    ) -> Result<Response<criapi::ReopenContainerLogResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_reopen_container_log(request).await;
        self.debug_response(&response);
        response
    }

    async fn exec_sync(
        &self,
        request: Request<criapi::ExecSyncRequest>,
    ) -> Result<Response<criapi::ExecSyncResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_exec_sync(request).await;
        self.debug_response(&response);
        response
    }

    async fn exec(
        &self,
        request: Request<criapi::ExecRequest>,
    ) -> Result<Response<criapi::ExecResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_exec(request).await;
        self.debug_response(&response);
        response
    }

    async fn attach(
        &self,
        request: Request<criapi::AttachRequest>,
    ) -> Result<Response<criapi::AttachResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_attach(request).await;
        self.debug_response(&response);
        response
    }
    async fn port_forward(
        &self,
        request: Request<criapi::PortForwardRequest>,
    ) -> Result<Response<criapi::PortForwardResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_port_forward(request).await;
        self.debug_response(&response);
        response
    }

    async fn run_pod_sandbox(
        &self,
        request: Request<criapi::RunPodSandboxRequest>,
    ) -> Result<Response<criapi::RunPodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_run_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn stop_pod_sandbox(
        &self,
        request: Request<criapi::StopPodSandboxRequest>,
    ) -> Result<Response<criapi::StopPodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_stop_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn remove_pod_sandbox(
        &self,
        request: Request<criapi::RemovePodSandboxRequest>,
    ) -> Result<Response<criapi::RemovePodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_remove_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn list_pod_sandbox(
        &self,
        request: Request<criapi::ListPodSandboxRequest>,
    ) -> Result<Response<criapi::ListPodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn pod_sandbox_status(
        &self,
        request: Request<criapi::PodSandboxStatusRequest>,
    ) -> Result<Response<criapi::PodSandboxStatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_pod_sandbox_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn status(
        &self,
        request: Request<criapi::StatusRequest>,
    ) -> Result<Response<criapi::StatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn update_runtime_config(
        &self,
        request: Request<criapi::UpdateRuntimeConfigRequest>,
    ) -> Result<Response<criapi::UpdateRuntimeConfigResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_update_runtime_config(request).await;
        self.debug_response(&response);
        response
    }
}
