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
use oci_spec::runtime::{LinuxBuilder, ProcessBuilder, RootBuilder, SpecBuilder, UserBuilder};
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
            container_id: container.id().into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cri::{
        api::{
            ContainerConfig, ContainerMetadata, CreateContainerRequest, Int64Value, KeyValue,
            LinuxContainerConfig, LinuxContainerSecurityContext, Mount,
        },
        cri_service::tests::new_cri_service,
    };
    use anyhow::Result;
    use std::collections::HashMap;

    fn create_request(config: Option<ContainerConfig>) -> Result<CreateContainerRequest> {
        let request = CreateContainerRequest {
            pod_sandbox_id: "123".to_owned(),
            sandbox_config: None,
            config,
        };

        Ok(request)
    }

    fn create_config(linux: Option<LinuxContainerConfig>) -> Result<ContainerConfig> {
        let tmp = tempfile::tempdir()?;

        let mut labels = HashMap::with_capacity(2);
        labels.insert("label1".to_owned(), "lvalue1".to_owned());
        labels.insert("label2".to_owned(), "lvalue2".to_owned());

        let mut annotations = HashMap::with_capacity(2);
        annotations.insert("annotation1".to_owned(), "avalue1".to_owned());
        annotations.insert("annotation2".to_owned(), "avalue2".to_owned());

        Ok(ContainerConfig {
            metadata: Some(ContainerMetadata {
                name: "vicious_tuna".to_owned(),
                attempt: 1,
            }),
            image: None,
            command: vec!["sleep".to_owned()],
            args: vec!["9000".to_owned()],
            working_dir: "/var/run/containrs".to_owned(),
            envs: vec![
                KeyValue {
                    key: "PATH".to_owned(),
                    value: "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
                        .to_owned(),
                },
                KeyValue {
                    key: "LOG_LEVEL".to_owned(),
                    value: "debug".to_owned(),
                },
            ],
            mounts: vec![Mount {
                container_path: "/path/in/container".to_owned(),
                host_path: tmp.into_path().to_string_lossy().to_string(),
                readonly: false,
                selinux_relabel: false,
                propagation: 0,
            }],
            devices: Vec::new(),
            labels,
            annotations,
            log_path: "/var/run/containrs/".to_owned(),
            stdin: false,
            stdin_once: false,
            tty: false,
            windows: None,
            linux,
        })
    }

    fn create_linux(
        security_context: Option<LinuxContainerSecurityContext>,
    ) -> LinuxContainerConfig {
        LinuxContainerConfig {
            resources: None,
            security_context,
        }
    }

    fn create_security_context() -> LinuxContainerSecurityContext {
        LinuxContainerSecurityContext {
            capabilities: None,
            privileged: false,
            run_as_user: Some(Int64Value { value: 1000 }),
            run_as_group: Some(Int64Value { value: 1000 }),
            supplemental_groups: vec![1000, 1001],
            run_as_username: "somebody".to_owned(),
            readonly_rootfs: false,
            apparmor_profile: "containrs_secure_profile".to_owned(),
            no_new_privs: true,
            masked_paths: vec!["/proc/kcore".to_owned()],
            readonly_paths: vec!["/proc/sys".to_owned()],
            namespace_options: None,
            seccomp_profile_path: "localhost/docker-default".to_owned(),
            selinux_options: None,
        }
    }

    #[tokio::test]
    async fn create_container_success() -> Result<()> {
        let sut = new_cri_service()?;
        let security_context = create_security_context();
        let linux_config = create_linux(Some(security_context));
        let config = create_config(Some(linux_config))?;
        let request = create_request(Some(config))?;

        let response = sut.handle_create_container(Request::new(request)).await?;
        assert_eq!(response.get_ref().container_id, "vicious_tuna.1".to_owned());
        Ok(())
    }

    #[tokio::test]
    async fn create_container_fail_no_metadata() -> Result<()> {
        let sut = new_cri_service()?;
        let security_context = create_security_context();
        let linux_config = create_linux(Some(security_context));
        let mut config = create_config(Some(linux_config))?;
        config.metadata = None;
        let request = create_request(Some(config))?;

        let response = sut.handle_create_container(Request::new(request)).await;
        assert!(response.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn create_container_fail_no_config() -> Result<()> {
        let sut = new_cri_service()?;
        let request = create_request(None)?;

        let response = sut.handle_create_container(Request::new(request)).await;
        assert!(response.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn create_container_fail_no_security() -> Result<()> {
        let sut = new_cri_service()?;
        let linux_config = create_linux(None);
        let config = create_config(Some(linux_config))?;
        let request = create_request(Some(config))?;

        let response = sut.handle_create_container(Request::new(request)).await;
        assert!(response.is_err());
        Ok(())
    }
}
