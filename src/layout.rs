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

pub fn load() -> PersistedShell {
    let path = path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return PersistedShell::default();
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| PersistedShell::default())
}

pub fn save(shell: &PersistedShell) {
    let path = path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(shell) {
        let _ = fs::write(path, raw);
    }
}

fn path() -> PathBuf {
    let mut path = config_root();
    path.push("maruzzella");
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
