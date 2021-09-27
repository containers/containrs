use std::convert::TryInto;

use crate::{
    cri::{
        api::{CreateContainerRequest, CreateContainerResponse},
        cri_service::{CRIService, OptionStatus, ResultStatus},
    },
    error::ServiceError,
};
use container::container::local::OCIContainerBuilder;
use container::container::Container;
use oci_spec::runtime::{ProcessBuilder, SpecBuilder};
use tonic::{Request, Response, Status};

use crate::cri::api::Mount as CRIMount;
use oci_spec::runtime::Mount as OCIMount;

impl CRIService {
    /// handle_create_container creates a new container in specified PodSandbox.
    pub async fn handle_create_container(
        &self,
        request: Request<CreateContainerRequest>,
    ) -> Result<Response<CreateContainerResponse>, Status> {
        let config = request
            .into_inner()
            .config
            .take()
            .ok_or_invalid("no container config provided")?;

        let metadata = config
            .metadata
            .ok_or_invalid("no container metadata provided")?;

        let spec = SpecBuilder::default()
            .process(
                ProcessBuilder::default()
                    .args(
                        config
                            .command
                            .into_iter()
                            .chain(config.args)
                            .collect::<Vec<String>>(),
                    )
                    .env(
                        config
                            .envs
                            .iter()
                            .map(|kv| format!("{}={}", kv.key, kv.value))
                            .collect::<Vec<String>>(),
                    )
                    .cwd(config.working_dir)
                    .build()
                    .map_internal("failed to build runtime spec process")?,
            )
            .mounts(
                prepare_mounts(&config.mounts)
                    .map_internal("failed to build oci runtime spec mounts")?,
            )
            .annotations(config.annotations)
            .build()
            .map_internal("failed to create runtime spec")?;

        let mut container = OCIContainerBuilder::default()
            .id(format!("{}.{}", metadata.name, metadata.attempt))
            .log_path(config.log_path)
            .spec(spec)
            .build()
            .map_internal("failed to build container")?;

        container
            .create()
            .await
            .map_internal("failed to create container")?;

        let resp = CreateContainerResponse {
            container_id: "stub".into(),
        };

        Ok(Response::new(resp))
    }
}

fn prepare_mounts(cri_mounts: &[CRIMount]) -> Result<Vec<OCIMount>, ServiceError> {
    let mut oci_mounts = cri_mounts
        .iter()
        .map(|m| m.try_into())
        .collect::<Result<Vec<OCIMount>, ServiceError>>()?;
    oci_mounts.append(&mut oci_spec::runtime::get_default_mounts());

    Ok(oci_mounts)
}
