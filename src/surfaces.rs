use serde::{Deserialize, Serialize};

use crate::spec::{SplitAxis, TabGroupSpec, WorkbenchNodeSpec};

pub const MAIN_SURFACE_ID: &str = "main";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceReturnTarget {
    pub surface_id: String,
    pub group_id: String,
    pub tab_index: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceWindowGeometry {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetachedWorkbenchSpec {
    pub id: String,
    pub workbench: WorkbenchNodeSpec,
    pub return_target: SurfaceReturnTarget,
    #[serde(default)]
    pub geometry: SurfaceWindowGeometry,
}

pub fn detach_tab(
    main: &mut WorkbenchNodeSpec,
    detached: &mut Vec<DetachedWorkbenchSpec>,
    source_surface_id: &str,
    source_group_id: &str,
    tab_id: &str,
    next_surface_id: &mut u64,
) -> Option<String> {
    let (mut tab, tab_index, appearance) = {
        let source = surface_workbench_mut(main, detached, source_surface_id)?;
        let group = find_group_mut(source, source_group_id)?;
        let tab_index = group.tabs.iter().position(|tab| tab.id == tab_id)?;
        let tab = group.tabs.remove(tab_index);
        repair_active_tab(group, tab_id);
        let appearance = (
            group.panel_appearance_id.clone(),
            group.panel_header_appearance_id.clone(),
            group.tab_strip_appearance_id.clone(),
            group.text_appearance_id.clone(),
        );
        normalize_workbench_node(source);
        (tab, tab_index, appearance)
    };

    let id = format!("detached-workbench-{}", *next_surface_id);
    *next_surface_id = next_surface_id.saturating_add(1);
    let root_group_id = format!("workbench-{id}-root");
    tab.panel_id = root_group_id.clone();
    detached.push(DetachedWorkbenchSpec {
        id: id.clone(),
        workbench: WorkbenchNodeSpec::Group(TabGroupSpec {
            id: root_group_id,
            active_tab_id: Some(tab.id.clone()),
            show_tab_strip: true,
            panel_appearance_id: appearance.0,
            panel_header_appearance_id: appearance.1,
            tab_strip_appearance_id: appearance.2,
            text_appearance_id: appearance.3,
            tabs: vec![tab],
        }),
        return_target: SurfaceReturnTarget {
            surface_id: source_surface_id.to_string(),
            group_id: source_group_id.to_string(),
            tab_index,
        },
        geometry: SurfaceWindowGeometry::default(),
    });
    Some(id)
}

pub fn move_tab_to_return_target(
    main: &mut WorkbenchNodeSpec,
    detached: &mut Vec<DetachedWorkbenchSpec>,
    source_surface_id: &str,
    source_group_id: &str,
    tab_id: &str,
) -> bool {
    let Some(surface_index) = detached
        .iter()
        .position(|surface| surface.id == source_surface_id)
    else {
        return false;
    };
    let target = detached[surface_index].return_target.clone();
    let mut tab = {
        let Some(group) = find_group_mut(&mut detached[surface_index].workbench, source_group_id)
        else {
            return false;
        };
        let Some(index) = group.tabs.iter().position(|tab| tab.id == tab_id) else {
            return false;
        };
        let tab = group.tabs.remove(index);
        repair_active_tab(group, tab_id);
        tab
    };
    normalize_workbench_node(&mut detached[surface_index].workbench);

    let destination = destination_workbench_mut(main, detached, &target.surface_id);
    let group = find_group_or_first_mut(destination, &target.group_id);
    let Some(group) = group else {
        return false;
    };
    tab.panel_id = group.id.clone();
    let index = target.tab_index.min(group.tabs.len());
    group.tabs.insert(index, tab);
    group.active_tab_id = Some(tab_id.to_string());
    true
}

pub fn merge_detached_surface(
    main: &mut WorkbenchNodeSpec,
    detached: &mut Vec<DetachedWorkbenchSpec>,
    surface_id: &str,
) -> bool {
    let Some(index) = detached.iter().position(|surface| surface.id == surface_id) else {
        return false;
    };
    let surface = detached.remove(index);
    let target_id = surface.return_target.surface_id.clone();
    let destination = destination_workbench_mut(main, detached, &target_id);

    match surface.workbench.clone() {
        WorkbenchNodeSpec::Group(mut returned) => {
            if let Some(group) =
                find_group_or_first_mut(destination, &surface.return_target.group_id)
            {
                let index = surface.return_target.tab_index.min(group.tabs.len());
                for (offset, mut tab) in returned.tabs.drain(..).enumerate() {
                    tab.panel_id = group.id.clone();
                    group.tabs.insert(index + offset, tab);
                }
                group.active_tab_id = group.tabs.get(index).map(|tab| tab.id.clone());
            } else {
                *destination = surface.workbench;
            }
        }
        returned => {
            if !graft_after_group(
                destination,
                &surface.return_target.group_id,
                returned.clone(),
            ) {
                let existing = std::mem::replace(
                    destination,
                    WorkbenchNodeSpec::Group(TabGroupSpec::new(
                        "workbench-merge-placeholder",
                        None,
                        Vec::new(),
                    )),
                );
                *destination = WorkbenchNodeSpec::Split {
                    axis: SplitAxis::Horizontal,
                    children: vec![existing, returned],
                };
            }
        }
    }

    let resolved_target = if target_id == surface_id {
        MAIN_SURFACE_ID
    } else {
        &target_id
    };
    for child in detached {
        if child.return_target.surface_id == surface_id {
            child.return_target.surface_id = resolved_target.to_string();
        }
    }
    true
}

fn destination_workbench_mut<'a>(
    main: &'a mut WorkbenchNodeSpec,
    detached: &'a mut [DetachedWorkbenchSpec],
    surface_id: &str,
) -> &'a mut WorkbenchNodeSpec {
    if surface_id != MAIN_SURFACE_ID {
        if let Some(index) = detached.iter().position(|surface| surface.id == surface_id) {
            return &mut detached[index].workbench;
        }
    }
    main
}

fn surface_workbench_mut<'a>(
    main: &'a mut WorkbenchNodeSpec,
    detached: &'a mut [DetachedWorkbenchSpec],
    surface_id: &str,
) -> Option<&'a mut WorkbenchNodeSpec> {
    if surface_id == MAIN_SURFACE_ID {
        Some(main)
    } else {
        detached
            .iter_mut()
            .find(|surface| surface.id == surface_id)
            .map(|surface| &mut surface.workbench)
    }
}

pub fn find_group_mut<'a>(
    node: &'a mut WorkbenchNodeSpec,
    id: &str,
) -> Option<&'a mut TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => (group.id == id).then_some(group),
        WorkbenchNodeSpec::Split { children, .. } => children
            .iter_mut()
            .find_map(|child| find_group_mut(child, id)),
    }
}

fn first_group_mut(node: &mut WorkbenchNodeSpec) -> Option<&mut TabGroupSpec> {
    match node {
        WorkbenchNodeSpec::Group(group) => Some(group),
        WorkbenchNodeSpec::Split { children, .. } => children.iter_mut().find_map(first_group_mut),
    }
}

fn find_group_or_first_mut<'a>(
    node: &'a mut WorkbenchNodeSpec,
    id: &str,
) -> Option<&'a mut TabGroupSpec> {
    if contains_group(node, id) {
        find_group_mut(node, id)
    } else {
        first_group_mut(node)
    }
}

fn contains_group(node: &WorkbenchNodeSpec, id: &str) -> bool {
    match node {
        WorkbenchNodeSpec::Group(group) => group.id == id,
        WorkbenchNodeSpec::Split { children, .. } => {
            children.iter().any(|child| contains_group(child, id))
        }
    }
}

fn graft_after_group(node: &mut WorkbenchNodeSpec, id: &str, returned: WorkbenchNodeSpec) -> bool {
    match node {
        WorkbenchNodeSpec::Group(group) if group.id == id => {
            let placeholder = TabGroupSpec::new("workbench-merge-placeholder", None, Vec::new());
            let existing = std::mem::replace(group, placeholder);
            *node = WorkbenchNodeSpec::Split {
                axis: SplitAxis::Horizontal,
                children: vec![WorkbenchNodeSpec::Group(existing), returned],
            };
            true
        }
        WorkbenchNodeSpec::Group(_) => false,
        WorkbenchNodeSpec::Split { axis, children } => {
            if *axis == SplitAxis::Horizontal {
                if let Some(index) = children.iter().position(
                    |child| matches!(child, WorkbenchNodeSpec::Group(group) if group.id == id),
                ) {
                    children.insert(index + 1, returned);
                    return true;
                }
            }
            children
                .iter_mut()
                .any(|child| graft_after_group(child, id, returned.clone()))
        }
    }
}

pub fn normalize_workbench_node(node: &mut WorkbenchNodeSpec) -> bool {
    match node {
        WorkbenchNodeSpec::Group(group) => group.tabs.is_empty(),
        WorkbenchNodeSpec::Split { children, .. } => {
            children.retain_mut(|child| !normalize_workbench_node(child));
            if children.len() == 1 {
                *node = children.remove(0);
                false
            } else {
                children.is_empty()
            }
        }
    }
}

fn repair_active_tab(group: &mut TabGroupSpec, removed: &str) {
    if group.active_tab_id.as_deref() == Some(removed) {
        group.active_tab_id = group.tabs.first().map(|tab| tab.id.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::text_tab;

    fn group(id: &str, tabs: &[&str]) -> WorkbenchNodeSpec {
        WorkbenchNodeSpec::Group(TabGroupSpec::new(
            id,
            tabs.first().copied(),
            tabs.iter()
                .map(|id| text_tab(id, id, id, id, true))
                .collect(),
        ))
    }

    fn ids(node: &mut WorkbenchNodeSpec, group_id: &str) -> Vec<String> {
        find_group_mut(node, group_id)
            .unwrap()
            .tabs
            .iter()
            .map(|tab| tab.id.clone())
            .collect()
    }

    #[test]
    fn detach_and_move_back_preserve_original_position() {
        let mut main = group("workbench-main", &["a", "b", "c"]);
        let mut detached = Vec::new();
        let mut next_id = 1;
        let surface_id = detach_tab(
            &mut main,
            &mut detached,
            MAIN_SURFACE_ID,
            "workbench-main",
            "b",
            &mut next_id,
        )
        .unwrap();
        assert_eq!(ids(&mut main, "workbench-main"), ["a", "c"]);
        let group_id = match &detached[0].workbench {
            WorkbenchNodeSpec::Group(group) => group.id.clone(),
            _ => unreachable!(),
        };
        assert!(move_tab_to_return_target(
            &mut main,
            &mut detached,
            &surface_id,
            &group_id,
            "b",
        ));
        assert_eq!(ids(&mut main, "workbench-main"), ["a", "b", "c"]);
    }

    #[test]
    fn closing_split_surface_grafts_tree_and_retargets_children() {
        let mut main = group("workbench-main", &["a"]);
        let mut detached = vec![
            DetachedWorkbenchSpec {
                id: "detached-workbench-1".into(),
                workbench: WorkbenchNodeSpec::Split {
                    axis: SplitAxis::Vertical,
                    children: vec![group("detached-a", &["b"]), group("detached-b", &["c"])],
                },
                return_target: SurfaceReturnTarget {
                    surface_id: MAIN_SURFACE_ID.into(),
                    group_id: "workbench-main".into(),
                    tab_index: 1,
                },
                geometry: SurfaceWindowGeometry::default(),
            },
            DetachedWorkbenchSpec {
                id: "detached-workbench-2".into(),
                workbench: group("detached-child", &["d"]),
                return_target: SurfaceReturnTarget {
                    surface_id: "detached-workbench-1".into(),
                    group_id: "detached-a".into(),
                    tab_index: 0,
                },
                geometry: SurfaceWindowGeometry::default(),
            },
        ];
        assert!(merge_detached_surface(
            &mut main,
            &mut detached,
            "detached-workbench-1",
        ));
        assert!(find_group_mut(&mut main, "detached-a").is_some());
        assert!(find_group_mut(&mut main, "detached-b").is_some());
        assert_eq!(detached[0].return_target.surface_id, MAIN_SURFACE_ID);
    }
}
