use crate::kubernetes::cri::{
    api::{RuntimeCondition, RuntimeStatus, StatusRequest, StatusResponse},
    cri_service::CRIService,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

impl CRIService {
    /// handle_status returns the status of the runtime.
    pub async fn handle_status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let resp = StatusResponse {
            status: Some(RuntimeStatus {
                conditions: vec![
                    RuntimeCondition {
                        r#type: "RuntimeReady".into(),
                        status: true,
                        reason: "".into(),
                        message: "".into(),
                    },
                    RuntimeCondition {
                        r#type: "NetworkReady".into(),
                        status: true,
                        reason: "".into(),
                        message: "".into(),
                    },
                ],
            }),
            info: HashMap::new(),
        };
        Ok(Response::new(resp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::cri::{
        api::runtime_service_server::RuntimeService, cri_service::tests::new_cri_service,
    };
    use anyhow::{Context, Result};

    #[tokio::test]
    async fn runtime_status() -> Result<()> {
        let sut = new_cri_service()?;
        let request = StatusRequest { verbose: true };
        let response = sut.status(Request::new(request)).await?;
        let conditions = response
            .into_inner()
            .status
            .context("no status")?
            .conditions;
        assert_eq!(conditions.len(), 2);
        let runtime_condition = conditions.get(0).context("no runtime condition")?;
        let network_condition = conditions.get(1).context("no network condition")?;
        assert_eq!(runtime_condition.r#type, "RuntimeReady");
        assert_eq!(runtime_condition.status, true);
        assert_eq!(network_condition.r#type, "NetworkReady");
        assert_eq!(network_condition.status, true);
        Ok(())
    }
}
