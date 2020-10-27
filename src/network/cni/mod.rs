//! A network implementation which does work with the Kubernetes Container Network Interface (CNI).

use crate::{
    network::{
        cni::{
            config::{Config, ConfigBuilder, ConfigFile, ConfigListFile},
            exec::{DefaultExec, Exec},
            namespace::Namespace,
            netlink::Netlink,
            plugin::{CNIResult, Plugin, PluginBuilder},
        },
        PodNetwork,
    },
    sandbox::SandboxData,
    storage::{default_key_value_storage::DefaultKeyValueStorage, KeyValueStorage},
};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use crossbeam_channel::Sender;
use derive_builder::Builder;
use getset::{Getters, MutGetters};
use log::{debug, error, info, trace, warn};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Error as NotifyError, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    result,
    sync::Arc,
};
use tokio::sync::RwLock;

mod config;
mod exec;
mod namespace;
mod netlink;
mod plugin;

#[derive(Builder, Default, Getters)]
#[builder(default, pattern = "owned", setter(into))]
/// The pod network implementation based on the Container Network Interface.
pub struct CNI {
    #[get]
    /// This is the default CNI network name set by the user.
    default_network_name: Option<String>,

    #[get]
    /// The paths to configuration files.
    config_paths: Vec<PathBuf>,

    #[get]
    /// The pugin search paths to be used.
    plugin_paths: String,

    #[get]
    /// The storage path to be used for persisting CNI results.
    storage_path: Option<PathBuf>,

    #[get]
    /// The configuration watcher for monitoring config path changes.
    watcher: Option<(RecommendedWatcher, Sender<WatcherMessage>)>,

    #[get]
    /// CNI network state.
    state: State,

    #[getset(get, set = "pub")]
    #[builder(default = "Some(Box::new(DefaultExec))")]
    /// CNI command execution helper.
    plugin_exec: Option<Box<dyn Exec>>,
}

/// State is the internal state for the CNI which can be shared across threads safely.
type State = Arc<RwLock<CNIState>>;

#[derive(Builder, Debug, Default, Getters, MutGetters)]
#[builder(default, pattern = "owned", setter(into))]
/// The CNI state which will be setup on the `initialize` method call.
pub struct CNIState {
    #[get]
    /// The current default CNI network.
    default_network: Option<Config>,

    #[get]
    /// This is the default CNI network name set by the user.
    default_network_name: Option<String>,

    #[getset(get, get_mut)]
    /// Configuration storage, referenced by their file path on disk.
    configs: HashMap<PathBuf, Config>,

    #[get]
    /// The plugin search paths to be used.
    plugin_paths: String,

    #[getset(get_mut)]
    /// The storage instance to be used for persisting CNI results.
    storage: Option<DefaultKeyValueStorage>,
}

/// Selector for watcher messages on the receiver channel.
pub enum WatcherMessage {
    Handle(result::Result<Event, NotifyError>),
    Exit,
}

impl CNI {
    /// Initialize the CNI network
    pub async fn initialize(&mut self) -> Result<()> {
        let path_bufs_to_string = |x: &[PathBuf]| -> String {
            x.iter()
                .map(|x| x.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        // Validate the config paths
        if self.config_paths().is_empty() {
            bail!("no config paths provided")
        }
        debug!(
            "Configuration paths: {}",
            path_bufs_to_string(self.config_paths())
        );

        // Validate the plugin paths
        if self.plugin_paths().is_empty() {
            bail!("no plugin paths provided")
        }
        debug!("Plugin paths: {}", self.plugin_paths());
        self.state().write().await.plugin_paths = self.plugin_paths().clone();

        // Load all network configurations
        info!("Initializing CNI network");
        match self.default_network_name() {
            None => info!("No default CNI network name, choosing first one"),
            Some(name) => info!("Using default network name: {}", name),
        };
        self.state().write().await.default_network_name = self.default_network_name().clone();
        self.load_networks().await.context("load network configs")?;

        // Setup the storage if a path is set
        match self.storage_path() {
            Some(path) => {
                trace!("Setup CNI storage in {}", path.display());
                self.state().write().await.storage =
                    Some(DefaultKeyValueStorage::open(path).context("open storage path")?)
            }
            None => warn!("Not using CNI storage, cannot persist network results"),
        }

        // Create a config watcher
        let (tx, rx) = crossbeam_channel::unbounded();
        let tx_clone = tx.clone();
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |event| {
            tx_clone
                .send(WatcherMessage::Handle(event))
                .expect("watcher died because cannot send to channel");
        })
        .context("create config watcher")?;
        for config_path in self.config_paths() {
            // Create the directory if not existing
            tokio::fs::create_dir_all(config_path)
                .await
                .with_context(|| {
                    format!(
                        "create not existing CNI config path {}",
                        config_path.display()
                    )
                })?;

            // Add the watcher
            watcher
                .watch(config_path, RecursiveMode::NonRecursive)
                .with_context(|| format!("watch path {}", config_path.display()))?;
        }
        self.watcher = Some((watcher, tx));

        // Spawn watching thread
        let state = self.state().clone();
        tokio::spawn(async move {
            loop {
                match rx.recv() {
                    Ok(WatcherMessage::Exit) => {
                        debug!("Stopped watcher thread");
                        return;
                    }
                    Ok(WatcherMessage::Handle(Ok(event))) => {
                        if let Err(e) = Self::handle_event(&state, event).await {
                            error!("Unable to handle event: {:#}", e)
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Handle a file watcher event.
    async fn handle_event(state: &State, event: Event) -> Result<()> {
        trace!("Got file watcher event: {:?}", &event);
        match (event.kind, event.paths.get(0), event.paths.get(1)) {
            // File creation handline
            (EventKind::Create(CreateKind::File), Some(file), None)
                if Self::is_config_file(&file) =>
            {
                info!("Created new CNI config file {}", file.display());
                let config = Self::load_network(&state, &file)
                    .await
                    .context("load config")?;
                Self::insert_config(state, config)
                    .await
                    .context("add config")?;
                Self::log_networks(state).await
            }

            // File renaming handling
            (EventKind::Modify(ModifyKind::Name(_)), Some(old), Some(new)) => {
                info!(
                    "Renamed CNI config file from {} to {}",
                    old.display(),
                    new.display()
                );
                if Self::has_config_file_extensions(&old) {
                    Self::remove_config(state, &old)
                        .await
                        .context("remove old config")?;
                }
                if Self::is_config_file(&new) {
                    let config = Self::load_network(&state, &new)
                        .await
                        .context("load new config")?;
                    Self::insert_config(state, config)
                        .await
                        .context("add new config")?;
                }
                Self::log_networks(state).await
            }

            // File removal handling
            (EventKind::Remove(RemoveKind::File), Some(file), None)
                if Self::has_config_file_extensions(&file) =>
            {
                Self::remove_config(state, &file)
                    .await
                    .context("remove config")?;
                Self::log_networks(state).await
            }

            _ => Ok(()),
        }
    }

    /// Load all networks from the `config_paths`.
    async fn load_networks(&mut self) -> Result<()> {
        let files = self.config_files().context("get CNI config files")?;
        debug!("Got network files: {:?}", files);

        for file in files {
            match Self::load_network(self.state(), &file).await {
                Err(e) => {
                    warn!("Unable to load network {}: {:#}", file.display(), e);
                    continue;
                }
                Ok(config) => Self::insert_config(self.state(), config).await?,
            }
        }

        Self::log_networks(self.state()).await?;
        Ok(())
    }

    /// Log the currently loaded networks by their name
    async fn log_networks(state: &State) -> Result<()> {
        let state = state.read().await;
        let len = state.configs().len();
        let mut networks = state
            .configs()
            .values()
            .map(|x| x.name().to_string())
            .collect::<Vec<_>>();
        networks.sort();
        if len == 0 {
            info!("No loaded networks")
        } else {
            info!(
                "Currently loaded {} network{}: {}",
                len,
                if len > 1 { "s" } else { "" },
                networks.join(", ")
            );
        }
        Ok(())
    }

    /// Load a single CNI network for the provided configuration path.
    async fn load_network(state: &State, file: &Path) -> Result<Config> {
        debug!("Loading network from file {}", file.display());
        let config_file = match file
            .extension()
            .context("no file extension")?
            .to_str()
            .context("convert os string")?
        {
            "conflist" => ConfigListFile::from(&file).context("load config list from file")?,
            _ => ConfigFile::from(&file)
                .context("load config from file")?
                .into(),
        };

        let name = config_file
            .name()
            .as_ref()
            .context("no config file name provided")?
            .to_string();

        let config = ConfigBuilder::default()
            .name(&name)
            .file(file)
            .list(config_file)
            .build()
            .context("build CNI config")?;

        debug!("Validating network config: {:?}", config);
        config
            .validate(state.read().await.plugin_paths())
            .await
            .context("validate CNI config")?;

        info!(
            "Found valid CNI network config {} (type {}) in {}",
            name,
            config
                .list()
                .plugins()
                .get(0)
                .context("no plugin in config list")?
                .typ(),
            file.display()
        );

        Ok(config)
    }

    /// Returns all config files in the previously set `config_paths`
    fn config_files(&self) -> Result<Vec<PathBuf>> {
        self.config_paths()
            .iter()
            .filter(|path| path.exists() && path.is_dir())
            .try_fold(vec![], |mut x, path| {
                x.append(
                    &mut fs::read_dir(path)
                        .with_context(|| format!("read config path {}", path.display()))?
                        .filter_map(Result::ok)
                        .map(|e| e.path())
                        .filter(|e| Self::is_config_file(e))
                        .collect::<Vec<_>>(),
                );
                Ok(x)
            })
            .map(|mut files| {
                files.sort();
                files
            })
    }

    /// Returns true if the file path is a possible config file.
    fn is_config_file(file: &Path) -> bool {
        file.is_file() && Self::has_config_file_extensions(file)
    }

    /// Returns true if the file path has config file extensions.
    fn has_config_file_extensions(file: &Path) -> bool {
        file.extension() == Some(OsStr::new("conf"))
            || file.extension() == Some(OsStr::new("conflist"))
            || file.extension() == Some(OsStr::new("json"))
    }

    /// Insert a new or update an existing config in the provided `configs` HashMap.
    async fn insert_config(state: &State, config: Config) -> Result<()> {
        trace!("Inserting/updating config {}", config.name());
        let mut state = state.write().await;

        // Set the default network if selected by name
        if let Some(name) = state.default_network_name() {
            if name == config.name() {
                info!("Found user selected default network {}", config.name());
                state.default_network = Some(config.clone());
            }
        } else {
            match state.default_network().as_ref() {
                // Set the default network name if not already specified and not selected by the
                // user.
                None => {
                    info!("Automatically setting default network to {}", config.name());
                    state.default_network = Some(config.clone());
                }
                // Update the config only if the path is alphabetically before it and not selected
                // by the user.
                Some(c) if config.file() < c.file() => {
                    info!("Switching default network to {}", config.name());
                    state.default_network = Some(config.clone());
                }
                _ => {}
            }
        }

        state.configs_mut().insert(config.file().into(), config);
        Ok(())
    }

    /// Remove a config for the provided file path.
    async fn remove_config(state: &State, file: &Path) -> Result<()> {
        let mut state = state.write().await;

        info!("Removing CNI config {}", file.display());
        state.configs_mut().remove(file);

        match state.default_network().as_ref() {
            Some(c) if file == c.file() => {
                info!("Removed default network");

                // Only select the next work if not user-specified.
                if state.default_network_name().is_none() {
                    // Take the next network
                    let mut v: Vec<_> = state.configs().values().collect();
                    v.sort_by_key(|&x| x.file());

                    match v.first() {
                        None => {
                            info!("No new default network available");
                            state.default_network = None;
                        }
                        Some(c) => {
                            info!(
                                "Using {} as new default network ({})",
                                c.name(),
                                c.file().display()
                            );
                            state.default_network = Some((*c).clone());
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Retrieve necessary data for network start/stop.
    async fn get_start_stop_data<'a, 'b>(
        &'b self,
        sandbox_data: &'a SandboxData,
    ) -> Result<(&'a Path, Namespace)> {
        let network_namespace_path = sandbox_data
            .network_namespace_path()
            .as_ref()
            .context("no network namespace path provided")?;

        let netns = Namespace::new(network_namespace_path)
            .await
            .context("create network namespace")?;

        Ok((network_namespace_path, netns))
    }

    /// Build a CNI plugin for start/stop execution.
    async fn build_plugin(&self, binary: &str) -> Result<Plugin> {
        let mut plugin = PluginBuilder::default()
            .binary(binary)
            .build()
            .context("build CNI plugin")?;
        plugin.set_exec(
            self.plugin_exec
                .as_ref()
                .context("no CNI plugin executor set")?
                .clone(),
        );
        Ok(plugin
            .find_binary(self.plugin_paths())
            .context("find plugin binary")?)
    }

    /// Get an `eth` prefixed interface name for the provided index.
    fn eth(i: usize) -> String {
        format!("eth{}", i)
    }
}

/// A value to be safed in the CNI network storage.
type StorageValues = Vec<StorageValue>;

#[derive(Builder, Default, Getters, Serialize, Deserialize)]
#[builder(default, pattern = "owned", setter(into))]
/// A single storage value.
struct StorageValue {
    #[get]
    /// CNI Plugin binary name.
    binary_name: String,

    #[get]
    /// Raw CNI config.
    raw_cni_config: Vec<u8>,

    #[get]
    /// The CNI add result.
    cni_result: CNIResult,
}

#[async_trait]
impl PodNetwork for CNI {
    /// Start a new network for the provided `SandboxData`.
    async fn start(&mut self, sandbox_data: &SandboxData) -> Result<()> {
        info!("Starting CNI network for sandbox {}", sandbox_data.id());
        // Things intentionally skipped for sake of initial implementation simplicity:
        //
        // - host port management
        // - ingress egress bandwidth handling via annotations

        let mut state = self.state.write().await;
        let (network_namespace_path, netns) = self.get_start_stop_data(sandbox_data).await?;

        let default_network = state
            .default_network()
            .as_ref()
            .context("no default network available")?;

        trace!("Setup loopback interface");
        netns
            .run(async move {
                let netlink = Netlink::new().await?;
                let loopback_link = netlink.loopback().await.context("get loopback link")?;
                netlink
                    .set_link_up(&loopback_link)
                    .await
                    .context("set loopback link up")
            })
            .await
            .context("init loopback interface")?;

        trace!("Adding networks via plugins");
        let mut value = vec![];
        for (i, config) in default_network.list().plugins().iter().enumerate() {
            // Add the network via the CNI plugin
            let cni_result = self
                .build_plugin(config.typ())
                .await?
                .add(
                    sandbox_data.id(),
                    &network_namespace_path.display().to_string(),
                    &Self::eth(i),
                    config.raw(),
                )
                .await
                .context("add network via plugin")?;

            value.push(
                StorageValueBuilder::default()
                    .binary_name(config.typ())
                    .raw_cni_config(config.raw().as_slice())
                    .cni_result(cni_result)
                    .build()
                    .context("build storage value")?,
            );
        }

        if let Some(storage) = state.storage_mut() {
            trace!("Storing CNI result");
            storage
                .insert(sandbox_data.id(), value)
                .context("insert CNI result into storage")?;
        }

        debug!("Started CNI network for sandbox {}", sandbox_data.id());
        Ok(())
    }

    /// Stop the network of the provided `SandboxData`.
    async fn stop(&mut self, sandbox_data: &SandboxData) -> Result<()> {
        info!("Stopping CNI network for sandbox {}", sandbox_data.id());
        let mut state = self.state.write().await;
        let (network_namespace_path, netns) = self.get_start_stop_data(sandbox_data).await?;

        trace!("Stopping loopback interface");
        netns
            .run(async move {
                let netlink = Netlink::new().await?;
                let loopback_link = netlink.loopback().await.context("get loopback link")?;
                netlink
                    .set_link_down(&loopback_link)
                    .await
                    .context("set loopback link down")
            })
            .await
            .context("tear down loopback interface")?;

        if let Some(storage) = state.storage_mut() {
            trace!("Removing all networks");

            let storage_values: StorageValues = storage
                .get(sandbox_data.id())
                .context("retrieve sandbox from storage")?
                .context("sandbox already removed")?;

            trace!(
                "Got {} networks for sandbox {}",
                storage_values.len(),
                sandbox_data.id()
            );

            for (i, value) in storage_values.iter().enumerate() {
                // Remove the network via the CNI plugin
                self.build_plugin(value.binary_name())
                    .await?
                    .del(
                        sandbox_data.id(),
                        &network_namespace_path.display().to_string(),
                        &Self::eth(i),
                        value.raw_cni_config(),
                    )
                    .await
                    .context("delete network via plugin")?;
            }

            trace!("Removing sandbox from storage");
            storage
                .remove(sandbox_data.id())
                .context("remove sandbox from storage")?;
        }

        debug!("Stopped CNI network for sandbox {}", sandbox_data.id());
        Ok(())
    }

    /// Cleanup the network on server shutdown.
    async fn cleanup(&mut self) -> Result<()> {
        trace!("Stopping watcher");
        self.watcher
            .as_ref()
            .context("no watcher set")?
            .1
            .send(WatcherMessage::Exit)
            .context("send exit signal to watcher thread")?;

        if let Some(storage) = self.state().write().await.storage_mut() {
            trace!("Persisting CNI storage");
            storage.persist().context("persist storage")?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn config_files() -> Result<()> {
        let temp_dir = TempDir::new()?.into_path();
        let d1 = temp_dir.join("1");
        let d2 = temp_dir.join("2");
        fs::create_dir(&d1)?;
        fs::create_dir(&d2)?;

        File::create(d1.join("cfg-3.json"))?;
        File::create(d1.join("cfg-4.conf"))?;
        File::create(d1.join("cfg-5.txt"))?;
        fs::create_dir(d1.join("some"))?;

        File::create(d2.join("cfg-1.other"))?;
        File::create(d2.join("cfg-2.conflist"))?;

        let mut cni = CNI::default();
        cni.config_paths = vec![d1.clone(), d2.clone()];

        let files = cni.config_files()?;
        assert_eq!(
            files,
            vec![
                d1.join("cfg-3.json"),
                d1.join("cfg-4.conf"),
                d2.join("cfg-2.conflist"),
            ]
        );

        Ok(())
    }

    #[tokio::test]
    async fn initialize_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cni = CNIBuilder::default()
            .default_network_name(Some("network".into()))
            .config_paths(vec![temp_dir.into_path()])
            .plugin_paths("c:d")
            .build()?;

        assert!(cni.default_network_name().is_some());
        assert_eq!(cni.config_paths().len(), 1);
        assert_eq!(cni.plugin_paths(), "c:d");
        assert_eq!(cni.state().read().await.configs().len(), 0);

        cni.initialize().await
    }
}
