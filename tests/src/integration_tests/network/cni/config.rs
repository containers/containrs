use crate::common::Sut;
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const LIST: &str = r#"{
    "cniVersion": "0.4.0",
    "name": "list",
    "plugins": [
        {
            "type": "bridge",
            "bridge": "cni-name0",
            "isGateway": true,
            "ipMasq": true,
            "hairpinMode": true,
            "ipam": {
            "type": "host-local",
            "routes": [{ "dst": "0.0.0.0/0" }],
            "ranges": [[{ "subnet": "10.88.0.0/16", "gateway": "10.88.0.1" }]]
            }
        },
        { "type": "portmap", "capabilities": { "portMappings": true } },
        { "type": "firewall" },
        { "type": "tuning" }
    ]
}"#;

const CONFIG: &str = r#"{
    "cniVersion": "0.4.0",
    "name": "config",
    "type": "bridge",
    "bridge": "cni0",
    "isGateway": true,
    "ipMasq": true,
    "hairpinMode": true,
    "ipam": {
        "type": "host-local",
        "routes": [
            { "dst": "0.0.0.0/0" },
            { "dst": "1100:200::1/24" }
        ],
        "ranges": [
            [{ "subnet": "10.85.0.0/16" }],
            [{ "subnet": "1100:200::/24" }]
        ]
    }
}"#;

fn add_config(path: &Path, content: &[u8]) -> Result<()> {
    let mut temp_path: PathBuf = path.into();
    temp_path.set_extension("bak");

    fs::write(temp_path.display().to_string(), content)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

#[tokio::test]
async fn cni_config_lifecycle_no_default_network() -> Result<()> {
    let mut sut = Sut::start().await?;
    assert!(sut.log_file_contains_line("Currently loaded 1 network: loopback")?);

    // New config list added
    let config_list = sut.cni_config_path().join("2-list.conflist");
    add_config(&config_list, LIST.as_bytes())?;
    assert!(sut.log_file_contains_line("Currently loaded 2 networks: list, loopback")?);

    // New config added
    let config = sut.cni_config_path().join("1-config.conf");
    add_config(&config, CONFIG.as_bytes())?;
    assert!(sut.log_file_contains_line("Currently loaded 3 networks: config, list, loopback")?);

    // Remove config
    fs::remove_file(config)?;
    assert!(sut.log_file_contains_line("Using list as new default network")?);

    // Remove list
    fs::remove_file(config_list)?;
    assert!(sut.log_file_contains_line("Using loopback as new default network")?);

    // Rename loopback
    let loopback = sut.cni_config_path().join("loopback.json");
    fs::rename(sut.cni_config_path().join("99-loopback.conf"), &loopback)?;
    assert!(sut.log_file_contains_line("Automatically setting default network to loopback")?);

    // Remove loopback
    fs::remove_file(loopback)?;
    assert!(sut.log_file_contains_line("No new default network available")?);

    sut.cleanup()
}

#[tokio::test]
async fn cni_config_lifecycle_with_default_network() -> Result<()> {
    let mut sut = Sut::start_with_args(vec!["--cni-default-network=list".into()]).await?;
    assert!(sut.log_file_contains_line("Using default network name: list")?);

    // New config list added
    let config_list = sut.cni_config_path().join("2-list.conflist");
    add_config(&config_list, LIST.as_bytes())?;
    assert!(sut.log_file_contains_line("Found user selected default network list")?);

    // New config added
    let config = sut.cni_config_path().join("1-config.conf");
    add_config(&config, CONFIG.as_bytes())?;
    assert!(sut.log_file_contains_line("Currently loaded 3 networks: config, list, loopback")?);

    // Remove list
    fs::remove_file(config_list)?;
    assert!(sut.log_file_contains_line("Removed default network")?);

    sut.cleanup()
}

#[tokio::test]
async fn cni_config_create_dir() -> Result<()> {
    let empty_temp_dir = TempDir::new()?;
    let mut sut = Sut::start_with_args(vec![format!(
        "--cni-config-paths={}",
        empty_temp_dir.path().join("new-dir").display()
    )])
    .await?;
    assert!(sut.log_file_contains_line("Currently loaded 1 network: loopback")?);

    sut.cleanup()?;
    Ok(())
}
