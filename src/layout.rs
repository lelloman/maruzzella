use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use maruzzella_api::MzConfigRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    let Ok(mut value) = serde_json::from_str::<Value>(&raw) else {
        return PersistedShell {
            spec: default_spec.clone(),
            panes: PanePositions::default(),
        };
    };
    inject_missing_tab_strip_flags(&mut value, default_spec);
    serde_json::from_value(value).unwrap_or_else(|_| PersistedShell {
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

fn inject_missing_tab_strip_flags(root: &mut Value, default_spec: &ShellSpec) {
    let Some(spec) = root.get_mut("spec").and_then(Value::as_object_mut) else {
        return;
    };

    merge_group_show_tab_strip(spec.get_mut("left_panel"), Some(&default_spec.left_panel));
    merge_group_show_tab_strip(spec.get_mut("right_panel"), Some(&default_spec.right_panel));
    merge_group_show_tab_strip(spec.get_mut("bottom_panel"), Some(&default_spec.bottom_panel));
    merge_workbench_show_tab_strip(spec.get_mut("workbench"), Some(&default_spec.workbench));
}

fn merge_group_show_tab_strip(current: Option<&mut Value>, default: Option<&crate::spec::TabGroupSpec>) {
    let (Some(current), Some(default)) = (current, default) else {
        return;
    };
    let Some(object) = current.as_object_mut() else {
        return;
    };
    if !object.contains_key("show_tab_strip") {
        object.insert(
            "show_tab_strip".to_string(),
            Value::Bool(default.show_tab_strip),
        );
    }
}

fn merge_workbench_show_tab_strip(
    current: Option<&mut Value>,
    default: Option<&crate::spec::WorkbenchNodeSpec>,
) {
    let (Some(current), Some(default)) = (current, default) else {
        return;
    };

    match (current, default) {
        (Value::Object(current_obj), crate::spec::WorkbenchNodeSpec::Group(default_group)) => {
            if let Some(group_value) = current_obj.get_mut("Group") {
                merge_group_show_tab_strip(Some(group_value), Some(default_group));
            }
        }
        (
            Value::Object(current_obj),
            crate::spec::WorkbenchNodeSpec::Split {
                children: default_children,
                ..
            },
        ) => {
            let Some(split_obj) = current_obj.get_mut("Split").and_then(Value::as_object_mut) else {
                return;
            };
            let Some(current_children) = split_obj.get_mut("children").and_then(Value::as_array_mut) else {
                return;
            };
            for (child, default_child) in current_children.iter_mut().zip(default_children.iter()) {
                merge_workbench_show_tab_strip(Some(child), Some(default_child));
            }
        }
        _ => {}
    }
}
