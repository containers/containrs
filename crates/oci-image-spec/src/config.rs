//! OpenContainer Config Specification
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "Cmd")]
    pub cmd: Option<Vec<String>>,
    #[serde(rename = "Entrypoint")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(rename = "Env")]
    pub env: Option<Vec<String>>,
    /// TODO: in original spec this is a map from string to object
    /// serde_json::Value is the best type for this field which i can guess
    #[serde(rename = "ExposedPorts")]
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "Labels")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(rename = "StopSignal")]
    pub stop_signal: Option<String>,
    #[serde(rename = "User")]
    pub user: Option<String>,
    #[serde(rename = "Volumes")]
    pub volumes: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "WorkingDir")]
    pub working_dir: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ItemHistory {
    pub author: Option<String>,
    pub comment: Option<String>,
    pub created: Option<String>,
    pub created_by: Option<String>,
    pub empty_layer: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ItemHistoryRootfs {
    pub diff_ids: Vec<String>,
    #[serde(rename = "type")]
    pub item_type: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct RootConfig {
    pub architecture: String,
    pub author: Option<String>,
    pub config: Option<Config>,
    pub created: Option<String>,
    pub history: Option<Vec<ItemHistory>>,
    pub os: String,
    pub rootfs: ItemHistoryRootfs,
}
