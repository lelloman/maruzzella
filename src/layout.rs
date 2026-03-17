use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use maruzzella_api::MzConfigRecord;
use serde::{Deserialize, Serialize};

use crate::product::default_product_spec;
use crate::spec::ShellSpec;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PanePositions {
    pub positions: HashMap<String, i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedShell {
    pub spec: ShellSpec,
    pub panes: PanePositions,
}

#[derive(Clone, Debug, Default)]
pub struct PluginConfigs {
    pub entries: HashMap<String, PluginConfigEntry>,
    pub invalid_entries: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginConfigEntry {
    pub schema_version: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StoredPluginConfigEntry {
    Legacy(Vec<u8>),
    Versioned(MzConfigRecord),
}

impl Default for PersistedShell {
    fn default() -> Self {
        Self {
            spec: default_product_spec().shell_spec(),
            panes: PanePositions::default(),
        }
    }
}

pub fn load(persistence_id: &str, default_spec: &ShellSpec) -> PersistedShell {
    let path = path(persistence_id);
    let Ok(raw) = fs::read_to_string(&path) else {
        return PersistedShell {
            spec: default_spec.clone(),
            panes: PanePositions::default(),
        };
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| PersistedShell {
        spec: default_spec.clone(),
        panes: PanePositions::default(),
    })
}

pub fn save(persistence_id: &str, shell: &PersistedShell) {
    let path = path(persistence_id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(shell) {
        let _ = fs::write(path, raw);
    }
}

pub fn path(persistence_id: &str) -> PathBuf {
    let mut path = config_root();
    path.push(persistence_id);
    path.push("layout.json");
    path
}

pub fn load_plugin_configs(persistence_id: &str) -> PluginConfigs {
    let path = plugin_configs_path(persistence_id);
    let Ok(raw) = fs::read_to_string(&path) else {
        return PluginConfigs::default();
    };
    let Ok(decoded) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) else {
        return PluginConfigs::default();
    };
    let mut configs = PluginConfigs::default();
    for (plugin_id, value) in decoded {
        match serde_json::from_value::<StoredPluginConfigEntry>(value) {
            Ok(StoredPluginConfigEntry::Legacy(payload)) => {
                configs.entries.insert(
                    plugin_id,
                    PluginConfigEntry {
                        schema_version: None,
                        payload,
                    },
                );
            }
            Ok(StoredPluginConfigEntry::Versioned(record)) => {
                configs.entries.insert(
                    plugin_id,
                    PluginConfigEntry {
                        schema_version: record.schema_version,
                        payload: record.payload,
                    },
                );
            }
            Err(error) => {
                configs
                    .invalid_entries
                    .insert(plugin_id, format!("stored plugin config is unreadable: {error}"));
            }
        }
    }
    configs
}

pub fn save_plugin_configs(persistence_id: &str, configs: &PluginConfigs) {
    let path = plugin_configs_path(persistence_id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let serializable = configs
        .entries
        .iter()
        .map(|(plugin_id, entry)| {
            (
                plugin_id.clone(),
                StoredPluginConfigEntry::Versioned(MzConfigRecord {
                    schema_version: entry.schema_version,
                    payload: entry.payload.clone(),
                }),
            )
        })
        .collect::<HashMap<_, _>>();
    if let Ok(raw) = serde_json::to_string_pretty(&serializable) {
        let _ = fs::write(path, raw);
    }
}

fn plugin_configs_path(persistence_id: &str) -> PathBuf {
    let mut path = config_root();
    path.push(persistence_id);
    path.push("plugins.json");
    path
}

fn config_root() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config");
        return path;
    }
    PathBuf::from(".")
}
