use std::convert::{TryFrom, TryInto};

use crate::{
    cri::{
        api::{CreateContainerRequest, CreateContainerResponse},
        cri_service::{CRIService, OptionStatus, ResultStatus},
    },
    error::ServiceError,
};
use container::container::local::OCIContainerBuilder;
use container::container::Container;
use oci_spec::runtime::{
    LinuxBuilder, ProcessBuilder, RootBuilder, SpecBuilder, UserBuilder,
};
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

        let linux_config = config
            .linux
            .ok_or_invalid("no container linux config provided")?;

        let security_context = linux_config
            .security_context
            .ok_or_invalid("no container security context provided")?;

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
                    .apparmor_profile(security_context.apparmor_profile)
                    .no_new_privileges(security_context.no_new_privs)
                    .user(
                        UserBuilder::default()
                            .uid(
                                u32::try_from(
                                    security_context
                                        .run_as_user
                                        .as_ref()
                                        .and_then(|id| Some(id.value))
                                        .unwrap_or_default(),
                                )
                                .map_internal("failed to convert uid")?,
                            )
                            .gid(
                                u32::try_from(
                                    security_context
                                        .run_as_group
                                        .as_ref()
                                        .and_then(|id| Some(id.value))
                                        .unwrap_or_default(),
                                )
                                .map_internal("failed to convert gid")?,
                            )
                            .additional_gids(
                                security_context
                                    .supplemental_groups
                                    .iter()
                                    .copied()
                                    .map(|id| u32::try_from(id))
                                    .collect::<Result<Vec<u32>, _>>()
                                    .map_internal("failed to convert supplemental groups")?,
                            )
                            .build()
                            .map_internal("failed to build runtime spec user")?,
                    )
                    .build()
                    .map_internal("failed to build runtime spec process")?,
            )
            .linux(
                LinuxBuilder::default()
                    .masked_paths(security_context.masked_paths)
                    .readonly_paths(security_context.readonly_paths)
                    .build()
                    .map_internal("failed to build runtime spec linux")?,
            )
            .root(
                RootBuilder::default()
                    .readonly(security_context.readonly_rootfs)
                    .build()
                    .map_internal("failed to build")?,
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
