//! Netlink related helpers and structures.

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use derive_builder::Builder;
use dyn_clone::{clone_trait_object, DynClone};
use futures_util::stream::TryStreamExt;
use getset::Getters;
use log::{debug, trace};
use netlink_packet_route::rtnl::RouteMessage;
use rtnetlink::{
    packet::rtnl::{link::nlas::Nla, LinkMessage},
    IpVersion,
};
use std::fmt;

#[async_trait]
/// The netlink behavior trait.
pub trait Netlink: DynClone + Send + Sync {
    /// Get the loopback link.
    async fn loopback(&self) -> Result<Link> {
        bail!("no loopback")
    }

    /// Get a link referenced by its name.
    async fn link_by_name(&self, _name: &str) -> Result<Link> {
        bail!("no link for name")
    }

    /// Get a link referenced by its index.
    async fn link_by_index(&self, _index: u32) -> Result<Link> {
        bail!("no link for index")
    }

    /// Set a link down.
    async fn set_link_down(&self, _link: &Link) -> Result<()> {
        Ok(())
    }

    /// Set a link up.
    async fn set_link_up(&self, _link: &Link) -> Result<()> {
        Ok(())
    }

    /// Get all routes for the provided IP version.
    async fn route_get(&self, _ip_version: IpVersion) -> Result<Vec<RouteMessage>> {
        Ok(vec![])
    }
}

clone_trait_object!(Netlink);

#[derive(Clone, Debug, Getters)]
/// The default Netlink interface implementation.
pub struct DefaultNetlink {
    #[get]
    handle: rtnetlink::Handle,
}

#[derive(Builder, Clone, Debug, Getters, Default)]
#[builder(default, pattern = "owned", setter(into))]
/// A link returned by netlink usage.
pub struct Link {
    #[get = "pub"]
    name: String,

    #[get = "pub"]
    message: LinkMessage,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl DefaultNetlink {
    /// Create a new netlink instance.
    pub async fn new() -> Result<Self> {
        debug!("Creating new netlink connection");

        let (connection, handle, _) =
            rtnetlink::new_connection().context("create new netlink connection")?;
        tokio::spawn(connection);

        Ok(Self { handle })
    }
}

#[async_trait]
impl Netlink for DefaultNetlink {
    /// Get the loopback link.
    async fn loopback(&self) -> Result<Link> {
        self.link_by_name("lo").await
    }

    /// Get a link referenced by its name.
    async fn link_by_name(&self, name: &str) -> Result<Link> {
        let link = Link {
            name: name.into(),
            message: self
                .handle()
                .link()
                .get()
                .set_name_filter(name.into())
                .execute()
                .try_next()
                .await
                .context("get links")?
                .with_context(|| format!("no link found for name {}", name))?,
        };
        trace!("Got link by name {}: {:?}", name, link.message.header);
        Ok(link)
    }

    /// Get a link referenced by its index.
    async fn link_by_index(&self, index: u32) -> Result<Link> {
        let message = self
            .handle()
            .link()
            .get()
            .match_index(index)
            .execute()
            .try_next()
            .await
            .context("get links")?
            .with_context(|| format!("no link found for index {}", index))?;
        trace!("Got link by index {}: {:?}", index, message.header);

        let name = || {
            for nla in message.nlas.iter() {
                if let Nla::IfName(name) = nla {
                    trace!("Found name {} for link index {}", name, index);
                    return Ok(name.clone());
                }
            }
            bail!("no name found for interface")
        };

        Ok(Link {
            name: name()?,
            message,
        })
    }

    /// Set a link down.
    async fn set_link_down(&self, link: &Link) -> Result<()> {
        trace!("Setting link {} down", link);
        self.handle()
            .link()
            .set(link.message().header.index)
            .down()
            .execute()
            .await
            .context("set link down")
    }

    /// Set a link up.
    async fn set_link_up(&self, link: &Link) -> Result<()> {
        trace!("Setting link {} up", link);
        self.handle()
            .link()
            .set(link.message().header.index)
            .up()
            .execute()
            .await
            .context("set link up")
    }

    /// Get all routes for the provided IP version.
    async fn route_get(&self, ip_version: IpVersion) -> Result<Vec<RouteMessage>> {
        self.handle()
            .route()
            .get(ip_version)
            .execute()
            .try_collect::<Vec<_>>()
            .await
            .context("get IP routes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn netlink() -> Result<()> {
        let netlink = DefaultNetlink::new().await?;
        netlink.loopback().await?;
        Ok(())
    }
}
