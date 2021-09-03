use anyhow::{Context, Result};
use cri::network::cni::port::{PortManager, PortMappingBuilder};
use ipnetwork::IpNetwork;
use log::info;
use nix::unistd::getuid;
use std::{net::SocketAddr, path::Path};
use tempfile::TempDir;
use tokio::process::Command;
use uuid::Uuid;

#[tokio::test]
async fn port_manager_ipv4() -> Result<()> {
    if !getuid().is_root() {
        info!("skipping IPv4 port manager test because not running as root");
        return Ok(());
    }

    let storage_path = TempDir::new()?;
    let mut port_manager = PortManager::new(storage_path.path()).await?;
    let id = new_id();

    // Add the port
    port_manager
        .add(
            &id,
            IpNetwork::V4("127.0.0.1/8".parse()?),
            &[&PortMappingBuilder::default()
                .host("127.0.0.1:8080".parse::<SocketAddr>()?)
                .container_port(8000u16)
                .protocol("tcp")
                .build()?],
        )
        .await?;

    // Verify
    let binary = which::which("iptables")?;
    let lines = test_iptables_std_output(&binary, &id).await?;
    assert_eq!(lines.len(), 5);
    assert!(lines
        .iter()
        .any(|x| x.contains("-m multiport --dports 8080 -j")));
    assert!(lines.get(0).context("no line 0")?.contains("-N"));
    assert!(lines.iter().any(|x| x.contains(
        "-s 127.0.0.0/8 -d 127.0.0.1/32 -p tcp -m tcp --dport 8080 -j CRI-HOSTPORT-SETMARK"
    )));
    assert!(lines.iter().any(|x| x.contains(
        "-s 127.0.0.1/32 -d 127.0.0.1/32 -p tcp -m tcp --dport 8080 -j CRI-HOSTPORT-SETMARK"
    )));
    assert!(lines.iter().any(|x| x.contains(
        "-d 127.0.0.1/32 -p tcp -m tcp --dport 8080 -j DNAT --to-destination 127.0.0.1:8000"
    )));

    // Remove the port
    port_manager.remove(&id).await?;

    // Verify
    let lines = test_iptables_std_output(&binary, &id).await?;
    assert!(lines.is_empty());

    Ok(())
}

#[tokio::test]
async fn port_manager_ipv6() -> Result<()> {
    if !getuid().is_root() {
        info!("skipping IPv6 port manager test because not running as root");
        return Ok(());
    }

    let storage_path = TempDir::new()?;
    let mut port_manager = PortManager::new(storage_path.path()).await?;
    let id = new_id();

    // Add the port
    port_manager
        .add(
            &id,
            IpNetwork::V6("::1/128".parse()?),
            &[&PortMappingBuilder::default()
                .host("[::1]:30080".parse::<SocketAddr>()?)
                .container_port(30090u16)
                .protocol("udp")
                .build()?],
        )
        .await?;

    // Verify
    let binary = which::which("ip6tables")?;
    let lines = test_iptables_std_output(&binary, &id).await?;
    assert!(lines.get(0).context("no line 0")?.contains("-N"));
    assert_eq!(lines.len(), 4);
    assert!(lines
        .iter()
        .any(|x| x.contains("-m multiport --dports 30080 -j")));
    assert!(lines.iter().any(
        |x| x.contains("-s ::1/128 -d ::1/128 -p udp -m udp --dport 30080 -j CRI-HOSTPORT-SETMARK")
    ));
    assert!(lines.iter().any(|x| x
        .contains("-d ::1/128 -p udp -m udp --dport 30080 -j DNAT --to-destination [::1]:30090")));

    // Remove the port
    port_manager.remove(&id).await?;

    // Verify
    let lines = test_iptables_std_output(&binary, &id).await?;
    assert!(lines.is_empty());

    Ok(())
}

fn new_id() -> String {
    let mut id = Uuid::new_v4().to_string();
    id.truncate(21);
    id
}

async fn test_iptables_std_output(binary: &Path, id: &str) -> Result<Vec<String>> {
    let output = Command::new(binary)
        .args(&["--wait", "-t", "nat", "-S"])
        .output()
        .await?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    assert!(stdout.contains("-N CRI-HOSTPORT-DNAT"));
    assert!(stdout.contains("-N CRI-HOSTPORT-MASQ"));
    assert!(stdout.contains("-N CRI-HOSTPORT-SETMARK"));
    assert!(stdout.contains("-A CRI-HOSTPORT-MASQ -m mark --mark 0x2000/0x2000 -j MASQUERADE"));
    assert!(stdout.contains("-A CRI-HOSTPORT-SETMARK -m comment --comment portforward-masquerade-mark -j MARK --set-xmark 0x2000/0x2000"));

    Ok(stdout
        .lines()
        .filter(|line| line.contains(id))
        .map(ToString::to_string)
        .collect::<Vec<String>>())
}
