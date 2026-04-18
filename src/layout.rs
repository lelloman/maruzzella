use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use maruzzella_api::MzConfigRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::product::default_product_spec;
use crate::spec::{ShellSpec, TabGroupSpec, WorkbenchNodeSpec};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PanePositions {
    pub positions: HashMap<String, i32>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub preferred_by_extent: HashMap<String, Vec<PaneExtentPreference>>,
    #[serde(default)]
    usage_clock: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaneExtentPreference {
    pub extent_bucket: i32,
    pub position: i32,
    #[serde(default = "default_preference_use_count")]
    pub use_count: u32,
    #[serde(default)]
    pub last_seen: u64,
}

impl PanePositions {
    pub fn has_preferred_position(&self, pane_id: &str, extent: i32) -> bool {
        pane_extent_bucket(extent)
            .and_then(|extent_bucket| {
                self.preferred_by_extent.get(pane_id).map(|entries| {
                    entries
                        .iter()
                        .any(|entry| entry.extent_bucket == extent_bucket)
                })
            })
            .unwrap_or(false)
    }

    pub fn preferred_position(&mut self, pane_id: &str, extent: i32) -> Option<i32> {
        let preferred = pane_extent_bucket(extent).and_then(|extent_bucket| {
            let next_seen = self.bump_usage_clock();
            self.preferred_by_extent
                .get_mut(pane_id)
                .and_then(|entries| {
                    entries
                        .iter_mut()
                        .find(|entry| entry.extent_bucket == extent_bucket)
                })
                .map(|entry| {
                    entry.use_count = entry.use_count.saturating_add(1);
                    entry.last_seen = next_seen;
                    entry.position
                })
        });
        preferred.or_else(|| self.positions.get(pane_id).copied())
    }

    pub fn remember_position(&mut self, pane_id: &str, extent: i32, position: i32) {
        self.positions.insert(pane_id.to_string(), position);

        let Some(extent_bucket) = pane_extent_bucket(extent) else {
            return;
        };

        let next_seen = self.bump_usage_clock();
        let entries = self
            .preferred_by_extent
            .entry(pane_id.to_string())
            .or_default();
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.extent_bucket == extent_bucket)
        {
            entry.position = position;
            entry.use_count = entry.use_count.saturating_add(1);
            entry.last_seen = next_seen;
        } else {
            entries.push(PaneExtentPreference {
                extent_bucket,
                position,
                use_count: 1,
                last_seen: next_seen,
            });
        }

        while entries.len() > max_pane_preferences() {
            let current_clock = self.usage_clock;
            let Some((lowest_index, _)) = entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| preference_score(entry, current_clock))
            else {
                break;
            };
            entries.remove(lowest_index);
        }
    }

    #[cfg(test)]
    fn tracked_buckets(&self, pane_id: &str) -> Vec<i32> {
        self.preferred_by_extent
            .get(pane_id)
            .map(|entries| entries.iter().map(|entry| entry.extent_bucket).collect())
            .unwrap_or_default()
    }

    fn bump_usage_clock(&mut self) -> u64 {
        self.usage_clock = self.usage_clock.saturating_add(1);
        self.usage_clock
    }
}

pub fn pane_extent_bucket(extent: i32) -> Option<i32> {
    if extent <= 0 {
        return None;
    }
    const BUCKET_SIZE: i32 = 64;
    Some(((extent + (BUCKET_SIZE / 2)) / BUCKET_SIZE) * BUCKET_SIZE)
}

fn default_preference_use_count() -> u32 {
    1
}

fn max_pane_preferences() -> usize {
    10
}

fn preference_score(entry: &PaneExtentPreference, current_clock: u64) -> i64 {
    let age = current_clock.saturating_sub(entry.last_seen) as i64;
    i64::from(entry.use_count) * 100 - age
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
    let mut shell =
        serde_json::from_value::<PersistedShell>(value).unwrap_or_else(|_| PersistedShell {
            spec: default_spec.clone(),
            panes: PanePositions::default(),
        });
    restore_app_owned_shell_fields(&mut shell.spec, default_spec);
    shell
}

pub fn load_for_slot(persistence_id: &str, slot: &str, default_spec: &ShellSpec) -> PersistedShell {
    let scoped = scoped_persistence_id(persistence_id, slot);
    let scoped_path = path(&scoped);
    if scoped_path.exists() {
        return load(&scoped, default_spec);
    }
    if slot == "workspace" {
        return load(persistence_id, default_spec);
    }
    PersistedShell {
        spec: default_spec.clone(),
        panes: PanePositions::default(),
    }
}

fn restore_app_owned_shell_fields(spec: &mut ShellSpec, default_spec: &ShellSpec) {
    spec.title = default_spec.title.clone();
    spec.search_placeholder = default_spec.search_placeholder.clone();
    spec.search_command_id = default_spec.search_command_id.clone();
    spec.status_text = default_spec.status_text.clone();
    spec.app_appearance_id = default_spec.app_appearance_id.clone();
    spec.topbar_appearance_id = default_spec.topbar_appearance_id.clone();
    spec.menu_appearance_id = default_spec.menu_appearance_id.clone();
    spec.toolbar_appearance_id = default_spec.toolbar_appearance_id.clone();
    spec.search_input_appearance_id = default_spec.search_input_appearance_id.clone();
    spec.status_appearance_id = default_spec.status_appearance_id.clone();
    spec.button_appearance_id = default_spec.button_appearance_id.clone();
    spec.text_appearance_id = default_spec.text_appearance_id.clone();
    spec.menu_roots = default_spec.menu_roots.clone();
    spec.menu_items = default_spec.menu_items.clone();
    spec.commands = default_spec.commands.clone();
    spec.toolbar_items = default_spec.toolbar_items.clone();
    restore_group_app_owned_fields(&mut spec.left_panel, &default_spec.left_panel);
    restore_group_app_owned_fields(&mut spec.right_panel, &default_spec.right_panel);
    restore_group_app_owned_fields(&mut spec.bottom_panel, &default_spec.bottom_panel);
    restore_workbench_app_owned_fields(&mut spec.workbench, &default_spec.workbench);
}

fn restore_group_app_owned_fields(group: &mut TabGroupSpec, default_group: &TabGroupSpec) {
    group.show_tab_strip = default_group.show_tab_strip;
    group.panel_appearance_id = default_group.panel_appearance_id.clone();
    group.panel_header_appearance_id = default_group.panel_header_appearance_id.clone();
    group.tab_strip_appearance_id = default_group.tab_strip_appearance_id.clone();
    group.text_appearance_id = default_group.text_appearance_id.clone();

    for tab in &mut group.tabs {
        if let Some(default_tab) = default_group
            .tabs
            .iter()
            .find(|candidate| candidate.id == tab.id)
        {
            tab.text_appearance_id = default_tab.text_appearance_id.clone();
        }
    }
}

fn restore_workbench_app_owned_fields(
    node: &mut WorkbenchNodeSpec,
    default_node: &WorkbenchNodeSpec,
) {
    match (node, default_node) {
        (WorkbenchNodeSpec::Group(group), WorkbenchNodeSpec::Group(default_group)) => {
            restore_group_app_owned_fields(group, default_group);
        }
        (
            WorkbenchNodeSpec::Split { children, .. },
            WorkbenchNodeSpec::Split {
                children: default_children,
                ..
            },
        ) => {
            for (child, default_child) in children.iter_mut().zip(default_children.iter()) {
                restore_workbench_app_owned_fields(child, default_child);
            }
        }
        _ => {}
    }
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

pub fn path_for_slot(persistence_id: &str, slot: &str) -> PathBuf {
    path(&scoped_persistence_id(persistence_id, slot))
}

pub fn scoped_persistence_id(persistence_id: &str, slot: &str) -> String {
    format!("{persistence_id}--{slot}")
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
                configs.invalid_entries.insert(
                    plugin_id,
                    format!("stored plugin config is unreadable: {error}"),
                );
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
    merge_group_show_tab_strip(
        spec.get_mut("bottom_panel"),
        Some(&default_spec.bottom_panel),
    );
    merge_workbench_show_tab_strip(spec.get_mut("workbench"), Some(&default_spec.workbench));
}

fn merge_group_show_tab_strip(
    current: Option<&mut Value>,
    default: Option<&crate::spec::TabGroupSpec>,
) {
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
            let Some(split_obj) = current_obj.get_mut("Split").and_then(Value::as_object_mut)
            else {
                return;
            };
            let Some(current_children) =
                split_obj.get_mut("children").and_then(Value::as_array_mut)
            else {
                return;
            };
            for (child, default_child) in current_children.iter_mut().zip(default_children.iter()) {
                merge_workbench_show_tab_strip(Some(child), Some(default_child));
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{restore_app_owned_shell_fields, PanePositions};
    use crate::product::default_product_spec;
    use crate::spec::{
        CommandSpec, MenuItemSpec, MenuRootSpec, TabGroupSpec, ToolbarDisplayMode, ToolbarItemSpec,
        WorkbenchNodeSpec,
    };

    #[test]
    fn pane_preferences_round_to_resolution_buckets() {
        let mut panes = PanePositions::default();
        panes.remember_position("shell.vertical", 1918, 1400);
        panes.remember_position("shell.vertical", 947, 640);

        assert_eq!(panes.preferred_position("shell.vertical", 1920), Some(1400));
        assert_eq!(panes.preferred_position("shell.vertical", 960), Some(640));
    }

    #[test]
    fn pane_preferences_fall_back_to_legacy_position() {
        let mut panes = PanePositions::default();
        panes.positions.insert("shell.outer".to_string(), 1200);

        assert_eq!(panes.preferred_position("shell.outer", 0), Some(1200));
        assert_eq!(panes.preferred_position("shell.outer", 1920), Some(1200));
    }

    #[test]
    fn pane_preferences_do_not_cross_resolution_buckets() {
        let mut panes = PanePositions::default();
        panes.remember_position("shell.horizontal", 1918, 320);
        panes.positions.insert("shell.horizontal".to_string(), 280);

        assert_eq!(panes.preferred_position("shell.horizontal", 960), Some(280));
    }

    #[test]
    fn pane_preferences_keep_frequent_resolutions_when_capped() {
        let mut panes = PanePositions::default();
        for _ in 0..5 {
            panes.remember_position("shell.vertical", 1920, 1400);
            assert_eq!(panes.preferred_position("shell.vertical", 1920), Some(1400));
        }

        for index in 0..10 {
            panes.remember_position("shell.vertical", 640 + (index * 64), 300 + index);
        }

        assert_eq!(panes.tracked_buckets("shell.vertical").len(), 10);
        assert_eq!(panes.preferred_position("shell.vertical", 1920), Some(1400));
        assert!(!panes.tracked_buckets("shell.vertical").contains(&640));
    }

    #[test]
    fn persisted_layout_does_not_override_app_owned_chrome() {
        let mut current = default_product_spec().shell_spec();
        current.title = "Current App".to_string();
        current.menu_roots = vec![MenuRootSpec {
            id: "file".to_string(),
            label: "File".to_string(),
        }];
        current.menu_items = vec![MenuItemSpec {
            id: "open".to_string(),
            root_id: "file".to_string(),
            label: "Open".to_string(),
            command_id: "app.open".to_string(),
            payload: Vec::new(),
        }];
        current.commands = vec![CommandSpec {
            id: "app.open".to_string(),
            title: "Open".to_string(),
        }];
        current.toolbar_items = vec![ToolbarItemSpec {
            id: "open".to_string(),
            icon_name: None,
            label: Some("Open".to_string()),
            command_id: "app.open".to_string(),
            payload: Vec::new(),
            secondary: false,
            display_mode: ToolbarDisplayMode::TextOnly,
            appearance_id: "primary".to_string(),
        }];
        current.workbench = WorkbenchNodeSpec::Group(
            TabGroupSpec::new("workbench-main", None, Vec::new())
                .with_tab_strip_appearance("current-workbench-tabs"),
        );

        let mut persisted = current.clone();
        persisted.title = "Stale App".to_string();
        persisted.status_text = "Stale status".to_string();
        persisted.menu_roots = vec![MenuRootSpec {
            id: "legacy".to_string(),
            label: "Legacy".to_string(),
        }];
        persisted.menu_items.clear();
        persisted.commands.clear();
        persisted.toolbar_items.clear();
        persisted.left_panel.panel_appearance_id = "stale-panel".to_string();
        persisted.left_panel.panel_header_appearance_id = "stale-header".to_string();
        persisted.left_panel.tab_strip_appearance_id = "stale-tabs".to_string();
        persisted.left_panel.text_appearance_id = "stale-text".to_string();
        persisted.workbench = WorkbenchNodeSpec::Group({
            let WorkbenchNodeSpec::Group(mut group) = current.workbench.clone() else {
                panic!("test workbench should be a group");
            };
            group.tab_strip_appearance_id = "stale-workbench-tabs".to_string();
            group
        });
        persisted.bottom_panel.active_tab_id = Some("persisted-bottom-tab".to_string());

        restore_app_owned_shell_fields(&mut persisted, &current);

        assert_eq!(persisted.title, "Current App");
        assert_eq!(persisted.status_text, current.status_text);
        assert_eq!(persisted.menu_roots[0].id, "file");
        assert_eq!(persisted.menu_items[0].id, "open");
        assert_eq!(persisted.commands[0].id, "app.open");
        assert_eq!(persisted.toolbar_items[0].id, "open");
        assert_eq!(
            persisted.left_panel.panel_appearance_id,
            current.left_panel.panel_appearance_id
        );
        assert_eq!(
            persisted.left_panel.panel_header_appearance_id,
            current.left_panel.panel_header_appearance_id
        );
        assert_eq!(
            persisted.left_panel.tab_strip_appearance_id,
            current.left_panel.tab_strip_appearance_id
        );
        assert_eq!(
            persisted.left_panel.text_appearance_id,
            current.left_panel.text_appearance_id
        );
        match (&persisted.workbench, &current.workbench) {
            (WorkbenchNodeSpec::Group(group), WorkbenchNodeSpec::Group(current_group)) => {
                assert_eq!(
                    group.tab_strip_appearance_id,
                    current_group.tab_strip_appearance_id
                );
            }
            _ => panic!("test workbench should be a group"),
        }
        assert_eq!(
            persisted.bottom_panel.active_tab_id.as_deref(),
            Some("persisted-bottom-tab")
        );
    }
}
