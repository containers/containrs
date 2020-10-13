use crate::kubernetes::cri::{
    api::{self, runtime_service_server::RuntimeService},
    cri_service::CRIService,
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
        request: Request<api::VersionRequest>,
    ) -> Result<Response<api::VersionResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_version(request).await;
        self.debug_response(&response);
        response
    }

    async fn create_container(
        &self,
        request: Request<api::CreateContainerRequest>,
    ) -> Result<Response<api::CreateContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_create_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn start_container(
        &self,
        request: Request<api::StartContainerRequest>,
    ) -> Result<Response<api::StartContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_start_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn stop_container(
        &self,
        request: Request<api::StopContainerRequest>,
    ) -> Result<Response<api::StopContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_stop_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn remove_container(
        &self,
        request: Request<api::RemoveContainerRequest>,
    ) -> Result<Response<api::RemoveContainerResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_remove_container(request).await;
        self.debug_response(&response);
        response
    }

    async fn list_containers(
        &self,
        request: Request<api::ListContainersRequest>,
    ) -> Result<Response<api::ListContainersResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_containers(request).await;
        self.debug_response(&response);
        response
    }

    async fn container_status(
        &self,
        request: Request<api::ContainerStatusRequest>,
    ) -> Result<Response<api::ContainerStatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_container_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn container_stats(
        &self,
        request: Request<api::ContainerStatsRequest>,
    ) -> Result<Response<api::ContainerStatsResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_container_stats(request).await;
        self.debug_response(&response);
        response
    }

    async fn list_container_stats(
        &self,
        request: Request<api::ListContainerStatsRequest>,
    ) -> Result<Response<api::ListContainerStatsResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_container_stats(request).await;
        self.debug_response(&response);
        response
    }

    async fn update_container_resources(
        &self,
        request: Request<api::UpdateContainerResourcesRequest>,
    ) -> Result<Response<api::UpdateContainerResourcesResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_update_container_resources(request).await;
        self.debug_response(&response);
        response
    }

    async fn reopen_container_log(
        &self,
        request: Request<api::ReopenContainerLogRequest>,
    ) -> Result<Response<api::ReopenContainerLogResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_reopen_container_log(request).await;
        self.debug_response(&response);
        response
    }

    async fn exec_sync(
        &self,
        request: Request<api::ExecSyncRequest>,
    ) -> Result<Response<api::ExecSyncResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_exec_sync(request).await;
        self.debug_response(&response);
        response
    }

    async fn exec(
        &self,
        request: Request<api::ExecRequest>,
    ) -> Result<Response<api::ExecResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_exec(request).await;
        self.debug_response(&response);
        response
    }

    async fn attach(
        &self,
        request: Request<api::AttachRequest>,
    ) -> Result<Response<api::AttachResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_attach(request).await;
        self.debug_response(&response);
        response
    }
    async fn port_forward(
        &self,
        request: Request<api::PortForwardRequest>,
    ) -> Result<Response<api::PortForwardResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_port_forward(request).await;
        self.debug_response(&response);
        response
    }

    async fn run_pod_sandbox(
        &self,
        request: Request<api::RunPodSandboxRequest>,
    ) -> Result<Response<api::RunPodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_run_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn stop_pod_sandbox(
        &self,
        request: Request<api::StopPodSandboxRequest>,
    ) -> Result<Response<api::StopPodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_stop_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn remove_pod_sandbox(
        &self,
        request: Request<api::RemovePodSandboxRequest>,
    ) -> Result<Response<api::RemovePodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_remove_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn list_pod_sandbox(
        &self,
        request: Request<api::ListPodSandboxRequest>,
    ) -> Result<Response<api::ListPodSandboxResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_list_pod_sandbox(request).await;
        self.debug_response(&response);
        response
    }

    async fn pod_sandbox_status(
        &self,
        request: Request<api::PodSandboxStatusRequest>,
    ) -> Result<Response<api::PodSandboxStatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_pod_sandbox_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn status(
        &self,
        request: Request<api::StatusRequest>,
    ) -> Result<Response<api::StatusResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_status(request).await;
        self.debug_response(&response);
        response
    }

    async fn update_runtime_config(
        &self,
        request: Request<api::UpdateRuntimeConfigRequest>,
    ) -> Result<Response<api::UpdateRuntimeConfigResponse>, Status> {
        self.debug_request(&request);
        let response = self.handle_update_runtime_config(request).await;
        self.debug_response(&response);
        response
    }
}
