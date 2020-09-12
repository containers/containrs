use crate::{
    cri_service::CRIService,
    criapi::{PortForwardRequest, PortForwardResponse},
};
use tonic::{Request, Response, Status};

impl CRIService {
    pub async fn handle_port_forward(
        &self,
        _request: Request<PortForwardRequest>,
    ) -> Result<Response<PortForwardResponse>, Status> {
        let resp = PortForwardResponse { url: "url".into() };
        Ok(Response::new(resp))
    }
}
