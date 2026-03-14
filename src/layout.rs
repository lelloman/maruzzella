use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
