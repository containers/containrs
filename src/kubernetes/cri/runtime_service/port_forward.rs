use crate::kubernetes::cri::{
    api::{PortForwardRequest, PortForwardResponse},
    cri_service::CRIService,
};
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_port_forward prepares a streaming endpoint to forward ports from a PodSandbox.
    pub async fn handle_port_forward(
        &self,
        _request: Request<PortForwardRequest>,
    ) -> Result<Response<PortForwardResponse>, Status> {
        let resp = PortForwardResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
