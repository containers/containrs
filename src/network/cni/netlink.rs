//! Netlink related helpers and structures.

use anyhow::{Context, Result};
use futures_util::stream::TryStreamExt;
use getset::Getters;
use log::{debug, trace};
use rtnetlink::packet::rtnl::LinkMessage;
use std::fmt;

#[derive(Clone, Debug)]
/// Netlink interface abstraction.
pub struct Netlink {
    handle: rtnetlink::Handle,
}

#[derive(Debug, Getters)]
/// A link returned by netlink usage.
pub struct Link {
    #[get]
    name: String,

    #[get]
    message: LinkMessage,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Netlink {
    /// Create a new netlink instance.
    pub async fn new() -> Result<Self> {
        debug!("Creating new netlink connection");

        let (connection, handle, _) =
            rtnetlink::new_connection().context("create new netlink connection")?;
        tokio::spawn(connection);

        Ok(Self { handle })
    }

    /// Get the loopback link.
    pub async fn loopback(&self) -> Result<Link> {
        self.link_by_name("lo").await
    }

    /// Get a link referenced by its name.
    pub async fn link_by_name(&self, name: &str) -> Result<Link> {
        let link = Link {
            name: name.into(),
            message: self
                .handle
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

    /// Set a link down.
    pub async fn set_link_down(&self, link: &Link) -> Result<()> {
        trace!("Setting link {} down", link);
        self.handle
            .link()
            .set(link.message().header.index)
            .down()
            .execute()
            .await
            .context("set link down")
    }

    /// Set a link up.
    pub async fn set_link_up(&self, link: &Link) -> Result<()> {
        trace!("Setting link {} up", link);
        self.handle
            .link()
            .set(link.message().header.index)
            .up()
            .execute()
            .await
            .context("set link up")
    }
}
