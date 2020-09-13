use crate::{
    criapi::{PortForwardRequest, PortForwardResponse},
    runtime_service::MyRuntime,
};
use tonic::{Request, Response, Status};

impl MyRuntime {
    pub async fn handle_port_forward(
        &self,
        _request: Request<PortForwardRequest>,
    ) -> Result<Response<PortForwardResponse>, Status> {
        let resp = PortForwardResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
