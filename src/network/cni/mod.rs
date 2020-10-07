//! A network implementation which does work with the Kubernetes Container Network Interface (CNI).

use crate::{
    error::chain,
    network::{
        cni::{
            config::{Config, ConfigBuilder, ConfigFile, ConfigListFile},
            exec::DefaultExec,
        },
        PodNetwork,
    },
};
use anyhow::{bail, format_err, Context, Result};
use crossbeam_channel::Sender;
use derive_builder::Builder;
use getset::{Getters, MutGetters};
use log::{debug, error, info, trace, warn};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Error as NotifyError, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    result,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

mod config;
mod exec;

#[derive(Builder, Default, Getters)]
#[builder(default, pattern = "owned", setter(into))]
/// The pod network implementation based on the Container Network Interface.
pub struct CNI {
    #[get]
    /// The network name of the default network to be used.
    default_network_name: Option<String>,

    #[get]
    /// The paths to configuration files.
    config_paths: Vec<PathBuf>,

    #[get]
    /// The paths to binary plugins.
    plugin_paths: Vec<PathBuf>,

    #[get]
    /// The configuration watcher for monitoring config path changes.
    watcher: Option<(RecommendedWatcher, Sender<WatcherMessage>)>,

    #[get]
    /// CNI network state.
    state: State,
}

#[derive(Clone, Debug, Default)]
/// State is the internal state for the CNI which can be shared across threads safely.
pub struct State(Arc<RwLock<CNIState>>);

impl State {
    /// Open the state in read-only mode.
    fn read(&self) -> Result<RwLockReadGuard<CNIState>> {
        self.0.read().map_err(|e| format_err!("read state: {}", e))
    }

    /// Open the state in read-write mode.
    fn write(&self) -> Result<RwLockWriteGuard<CNIState>> {
        self.0
            .write()
            .map_err(|e| format_err!("write state: {}", e))
    }
}

#[derive(Builder, Debug, Default, Getters, MutGetters)]
#[builder(default, pattern = "owned", setter(into))]
pub struct CNIState {
    #[get]
    /// The current default CNI network.
    default_network: Option<DefaultConfig>,

    #[get]
    /// Indicates if the default CNI network name can change or is user defined.
    default_network_name: Option<String>,

    #[getset(get, get_mut)]
    /// Configuration storage, referenced by their file path on disk.
    configs: HashMap<PathBuf, DefaultConfig>,
}

/// DefaultConfig is a config with a default exec.
type DefaultConfig = Config<DefaultExec>;

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
        debug!("Plugin paths: {}", path_bufs_to_string(self.plugin_paths()));

        // Load all network configurations
        info!("Initializing CNI network");
        match self.default_network_name() {
            None => info!("No default CNI network name, choosing first one"),
            Some(name) => info!("Using default network name: {}", name),
        };
        self.state.write()?.default_network_name = self.default_network_name().clone();
        self.load_networks().await.context("load network configs")?;

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
        let state = self.state.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv() {
                    Ok(WatcherMessage::Exit) => {
                        debug!("Stopped watcher thread");
                        return;
                    }
                    Ok(WatcherMessage::Handle(Ok(event))) => {
                        if let Err(e) = Self::handle_event(&state, event).await {
                            error!("Unable to handle event: {}", chain(e))
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
                let config = Self::load_network(&file).await.context("load config")?;
                Self::insert_config(state, config).context("add config")?;
                Self::log_networks(state)
            }

            // File renaming handling
            (EventKind::Modify(ModifyKind::Name(_)), Some(old), Some(new)) => {
                info!(
                    "Renamed CNI config file from {} to {}",
                    old.display(),
                    new.display()
                );
                if Self::has_config_file_extensions(&old) {
                    Self::remove_config(state, &old).context("remove old config")?;
                }
                if Self::is_config_file(&new) {
                    let config = Self::load_network(&new).await.context("load new config")?;
                    Self::insert_config(state, config).context("add new config")?;
                }
                Self::log_networks(state)
            }

            // File removal handling
            (EventKind::Remove(RemoveKind::File), Some(file), None)
                if Self::has_config_file_extensions(&file) =>
            {
                Self::remove_config(state, &file).context("remove config")?;
                Self::log_networks(state)
            }

            _ => Ok(()),
        }
    }

    /// Load all networks from the `config_paths`.
    async fn load_networks(&mut self) -> Result<()> {
        let files = self.config_files().context("get CNI config files")?;
        debug!("Got network files: {:?}", files);

        for file in files {
            match Self::load_network(&file).await {
                Err(e) => {
                    warn!("Unable to load network {}: {}", file.display(), chain(e));
                    continue;
                }
                Ok(config) => Self::insert_config(&self.state, config)?,
            }
        }

        Self::log_networks(&self.state)?;
        Ok(())
    }

    /// Log the currently loaded networks by their name
    fn log_networks(state: &State) -> Result<()> {
        let state = state.read()?;
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
    async fn load_network(file: &Path) -> Result<DefaultConfig> {
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
        config.validate().await.context("validate CNI config")?;

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
    fn insert_config(state: &State, config: DefaultConfig) -> Result<()> {
        trace!("Inserting/updating config {}", config.name());
        let mut state = state.write()?;

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
    fn remove_config(state: &State, file: &Path) -> Result<()> {
        let mut state = state.write()?;

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
}

impl PodNetwork for CNI {
    /// Cleanup the network on server shutdown.
    fn cleanup(&mut self) -> Result<()> {
        trace!("Stopping watcher");
        self.watcher
            .as_ref()
            .context("no watcher set")?
            .1
            .send(WatcherMessage::Exit)
            .context("send exit signal to watcher thread")?;
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
            .plugin_paths(["c", "d"].iter().map(PathBuf::from).collect::<Vec<_>>())
            .build()?;

        assert!(cni.default_network_name().is_some());
        assert_eq!(cni.config_paths().len(), 1);
        assert_eq!(cni.plugin_paths().len(), 2);
        assert_eq!(cni.state().read()?.configs().len(), 0);

        cni.initialize().await
    }
}
