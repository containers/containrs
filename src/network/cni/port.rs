//! Host to container port mapping functionality.

use crate::{
    network::cni::{
        iptables::{Chain, ChainBuilder, DefaultIptablesBuilder, Iptables},
        netlink::{DefaultNetlink, Netlink},
    },
    storage::{default_key_value_storage::DefaultKeyValueStorage, KeyValueStorage},
};
use anyhow::{format_err, Context, Result};
use derive_builder::Builder;
use getset::{CopyGetters, Getters, MutGetters};
use ipnetwork::IpNetwork;
use log::trace;
use rtnetlink::IpVersion;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::{IpAddr, SocketAddr},
    path::Path,
};
use sysctl::{Ctl, Sysctl};

#[derive(Getters, MutGetters)]
/// The main interface to manage port mappings.
pub struct PortManager {
    #[getset(get, get_mut)]
    /// Internal iptables instance to be used.
    iptables: Box<dyn Iptables>,

    #[get]
    /// Internal netlink instance to be used.
    netlink: Box<dyn Netlink>,

    #[getset(get, get_mut)]
    /// Storage used for tracking port mappings.
    storage: DefaultKeyValueStorage,
}

#[derive(Builder, Debug, CopyGetters, Getters, Hash)]
#[builder(pattern = "owned", setter(into))]
/// A PortMapping represents a host to container port connection.
pub struct PortMapping {
    #[get = "pub"]
    /// Host socket address to be used.
    host: SocketAddr,

    #[get_copy = "pub"]
    /// The port number inside the container.
    container_port: u16,

    #[get = "pub"]
    /// The protocol of the port mapping.
    protocol: String,
}

#[derive(Builder, Debug, CopyGetters, Getters, Serialize, Deserialize)]
#[builder(pattern = "owned", setter(into))]
/// A value saved in the internal storage for state tracking.
struct StorageValue {
    #[get]
    /// The DNAT iptables chain name stored.
    dnat_chain_name: String,

    #[get_copy]
    /// Indicator whether to use IPv6 or not.
    is_ipv6: bool,
}

impl PortManager {
    /// Create a new port manager instance.
    pub async fn new<P>(storage_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            netlink: Box::new(
                DefaultNetlink::new()
                    .await
                    .context("create netlink instance")?,
            ),
            storage: DefaultKeyValueStorage::open(storage_path).context("open storage path")?,
            iptables: Box::new(
                DefaultIptablesBuilder::default()
                    .iptables_binary(
                        which::which("iptables").context("find iptables binary in $PATH")?,
                    )
                    .ip6tables_binary(
                        which::which("ip6tables").context("find ip6tables binary in $PATH")?,
                    )
                    .build()
                    .context("create iptables instance")?,
            ),
        })
    }

    /// Add a port mappings for the provided network to the manager.
    pub async fn add<I>(
        &mut self,
        id: I,
        container_network: IpNetwork,
        port_mappings: &[&PortMapping],
    ) -> Result<()>
    where
        I: AsRef<str>,
    {
        let id = id.as_ref();
        trace!(
            "Adding port mappings for ID {} and network {}: {:?}",
            id,
            container_network,
            port_mappings
        );

        // Decide whether to use IPv6 or not.
        self.iptables_mut().set_ipv6(container_network.is_ipv6());

        // Enable masquerading for traffic as necessary.
        // The DNAT chain sets a mark bit for traffic that needs masq:
        // - connections from localhost
        // - hairpin traffic back to the container
        // Idempotently create the rule that masquerades traffic with this mark.
        // Need to do this first; the DNAT rules reference these chains
        let set_mark_chain = ChainBuilder::default()
            .name(Chain::SET_MARK_CHAIN_NAME)
            .rules(vec![vec![
                "-m".into(),
                "comment".into(),
                "--comment".into(),
                "portforward-masquerade-mark".into(),
                "-j".into(),
                "MARK".into(),
                "--set-xmark".into(),
                "0x2000/0x2000".into(),
            ]])
            .build()
            .context("build set mark chain")?;

        trace!("Setup set mark chain");
        self.iptables()
            .setup(&set_mark_chain)
            .await
            .context("setup set mark chain")?;

        let mark_masq_chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["POSTROUTING".into()])
            .entry_rules(vec![vec![
                "-m".into(),
                "comment".into(),
                "--comment".into(),
                "portforward-requiring-masquerade".into(),
            ]])
            .rules(vec![vec![
                "-m".into(),
                "mark".into(),
                "--mark".into(),
                "0x2000/0x2000".into(),
                "-j".into(),
                "MASQUERADE".into(),
            ]])
            .build()
            .context("build mark masq chain")?;

        trace!("Setup mark masq chain");
        self.iptables()
            .setup(&mark_masq_chain)
            .await
            .context("setup mark masq chain")?;

        if container_network.is_ipv4() {
            trace!("Trying to enable localnet routing for IPv4");

            // Set the route_localnet bit on the host interface, so that 127/8 can cross a routing
            // boundary.
            if let Some(interface_name) = self
                .get_routable_host_interface(container_network.ip())
                .await
                .context("get routable host interface")?
            {
                self.enable_localnet_routing(&interface_name)
                    .with_context(|| {
                        format!("enable localnet routing for interface {}", interface_name)
                    })?;
            }
        }

        // The top-level summary chain that we'll add our chain to.
        let top_level_dnat_chain = ChainBuilder::default()
            .name(Chain::TOP_LEVEL_DNAT_CHAIN_NAME)
            .entry_chains(vec!["PREROUTING".into(), "OUTPUT".into()])
            .entry_rules(vec![vec![
                "-m".into(),
                "addrtype".into(),
                "--dst-type".into(),
                "LOCAL".into(),
            ]])
            .build()
            .context("build top level DNAT chain")?;

        trace!("Setup top level DNAT chain");
        self.iptables()
            .setup(&top_level_dnat_chain)
            .await
            .context("setup top level DNAT chain")?;

        let dnat_chain_name = Self::hash(id, &(container_network, &port_mappings));
        let mut dnat_chain = ChainBuilder::default()
            .name(&dnat_chain_name)
            .entry_chains(vec![top_level_dnat_chain.name().into()])
            .build()
            .context("build DNAT chain")?;

        trace!("Filling DNAT rules");
        dnat_chain.fill_dnat_rules(&port_mappings, container_network);

        trace!("Setup DNAT chain");
        self.iptables
            .setup(&dnat_chain)
            .await
            .context("setup DNAT chain")?;

        // Save the result in the storage
        trace!("Inserting port mapping into storage for ID {}", id);
        self.storage_mut()
            .insert(
                id,
                StorageValueBuilder::default()
                    .dnat_chain_name(dnat_chain_name)
                    .is_ipv6(container_network.is_ipv6())
                    .build()
                    .context("build storage value")?,
            )
            .context("insert result into storage")
    }

    /// Remove a port mapping for the provided id.
    pub async fn remove(&mut self, id: &str) -> Result<()> {
        trace!("Removing port mappings for ID {}", id,);

        // Restore the result from the storage
        let storage_value: StorageValue = self
            .storage()
            .get(id)
            .context("retrieve result from storage")?
            .context("ID not in storage")?;
        trace!("Got storage value: {:?}", storage_value);

        // Decide whether to use IPv6 or not.
        self.iptables_mut().set_ipv6(storage_value.is_ipv6());

        let dnat_chain = ChainBuilder::default()
            .name(storage_value.dnat_chain_name())
            .entry_chains(vec![Chain::TOP_LEVEL_DNAT_CHAIN_NAME.into()])
            .build()
            .context("build DNAT chain")?;

        trace!("Removing iptables rules");
        self.iptables()
            .teardown(&dnat_chain)
            .await
            .context("teardown DNAT chain")
    }

    /// Hash the input type to a prefixed string.
    fn hash<T: Hash>(prefix: &str, t: &T) -> String {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);

        // limit the overall hash string to 28 characters
        let mut hash = format!("DN-CRI-{}-{:x}", prefix, s.finish());
        hash.truncate(28);
        hash
    }

    /// Tries to determine which interface routes the container's traffic. This is the one on which
    /// we disable martian filtering.
    async fn get_routable_host_interface(&self, ip: IpAddr) -> Result<Option<String>> {
        trace!("Getting routable host interface");
        let ip_version = if ip.is_ipv4() {
            IpVersion::V4
        } else {
            IpVersion::V6
        };
        let routes = self.netlink().route_get(ip_version).await?;
        trace!("Got {} routes", routes.len());

        for route in routes {
            if let Some(interface_id) = route.output_interface() {
                if let Ok(link) = self.netlink().link_by_index(interface_id).await {
                    trace!("Found routable output interface link: {}", link.name());
                    return Ok(Some(link.name().into()));
                }
            }
        }

        trace!("No routable host interface link found");
        Ok(None)
    }

    /// Tells the kernel not to treat 127/8 as a martian, so that connections with a source ip of
    /// 127/8 can cross a routing boundary.
    fn enable_localnet_routing(&self, interface_name: &str) -> Result<()> {
        trace!("Enabling localnet routing");
        let key = format!("net.ipv4.conf.{}.route_localnet", interface_name);
        let ctl = Ctl::new(&key).map_err(|e| format_err!("get sysctl {}: {}", key, e))?;
        ctl.set_value_string("1")
            .map_err(|e| format_err!("set sysctl {}: {}", key, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::cni::netlink::{Link, LinkBuilder};
    use anyhow::bail;
    use async_trait::async_trait;
    use netlink_packet_route::rtnl::{route::nlas::Nla, RouteHeader, RouteMessage};
    use std::net::{Ipv4Addr, Ipv6Addr};
    use tempfile::TempDir;

    #[derive(Clone, Debug, Default, Getters)]
    struct NetlinkMock {
        #[get]
        route_get_result: Option<Vec<RouteMessage>>,

        #[get]
        link_by_index_result: Option<Link>,
    }

    #[async_trait]
    impl Netlink for NetlinkMock {
        async fn route_get(&self, _ip_version: IpVersion) -> Result<Vec<RouteMessage>> {
            match self.route_get_result() {
                Some(vec) => Ok(vec.clone()),
                None => bail!("no result"),
            }
        }

        async fn link_by_index(&self, _index: u32) -> Result<Link> {
            match self.link_by_index_result() {
                Some(link) => Ok(link.clone()),
                None => bail!("no result"),
            }
        }
    }

    #[derive(Clone, Debug)]
    struct IptablesMock;

    impl Iptables for IptablesMock {}

    fn port_manager(
        route_get_result: Option<Vec<RouteMessage>>,
        link_by_index_result: Option<Link>,
    ) -> Result<PortManager> {
        let temp_dir = TempDir::new()?;
        Ok(PortManager {
            iptables: Box::new(IptablesMock),
            netlink: Box::new(NetlinkMock {
                route_get_result,
                link_by_index_result,
            }),
            storage: DefaultKeyValueStorage::open(temp_dir.path())?,
        })
    }

    #[tokio::test]
    async fn add_remove_success_ipv6() -> Result<()> {
        let mut port_manager = port_manager(None, None)?;
        port_manager
            .add(
                "id",
                IpNetwork::V6("ff01::0".parse()?),
                &[&PortMappingBuilder::default()
                    .host("[::1]:6000".parse::<SocketAddr>()?)
                    .container_port(6001u16)
                    .protocol("tcp")
                    .build()?],
            )
            .await?;
        port_manager.remove("id").await
    }

    #[tokio::test]
    async fn add_success_ipv4() -> Result<()> {
        let mut port_manager = port_manager(
            Some(vec![RouteMessage {
                header: RouteHeader::default(),
                nlas: vec![Nla::Oif(0)],
            }]),
            None,
        )?;
        port_manager
            .add(
                "id",
                IpNetwork::V4("127.0.0.1".parse()?),
                &[&PortMappingBuilder::default()
                    .host("[::1]:6000".parse::<SocketAddr>()?)
                    .container_port(6001u16)
                    .protocol("tcp")
                    .build()?],
            )
            .await?;
        port_manager.remove("id").await
    }

    #[tokio::test]
    async fn get_routable_host_interface_success_ipv4() -> Result<()> {
        let port_manager = port_manager(
            Some(vec![RouteMessage {
                header: RouteHeader::default(),
                nlas: vec![Nla::Oif(0)],
            }]),
            Some(LinkBuilder::default().build()?),
        )?;
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let res = port_manager.get_routable_host_interface(ip).await?;
        assert!(res.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn get_routable_host_interface_failure_ipv4_route_get() -> Result<()> {
        let port_manager = port_manager(None, None)?;
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert!(port_manager.get_routable_host_interface(ip).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn get_routable_host_interface_success_ipv6() -> Result<()> {
        let port_manager = port_manager(
            Some(vec![RouteMessage {
                header: RouteHeader::default(),
                nlas: vec![Nla::Oif(0)],
            }]),
            Some(LinkBuilder::default().build()?),
        )?;
        let ip = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let res = port_manager.get_routable_host_interface(ip).await?;
        assert!(res.is_some());
        Ok(())
    }

    #[test]
    fn hash() -> Result<()> {
        let port_mapping = vec![PortMappingBuilder::default()
            .host("101.10.15.3:8080".parse::<SocketAddr>()?)
            .container_port(30080u16)
            .protocol("tcp")
            .build()?];
        assert_eq!(
            &PortManager::hash("id", &port_mapping),
            "DN-CRI-id-3cc34f23f5021df2"
        );
        Ok(())
    }
}
