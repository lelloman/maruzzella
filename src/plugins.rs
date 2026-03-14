use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str;

use libloading::{Library, Symbol};
use maruzzella_api::{
    MzPluginDependency, MzPluginDescriptorView, MzPluginVTable, MzStr, MZ_ABI_VERSION_V1,
};

const ENTRY_SYMBOL: &[u8] = b"maruzzella_plugin_entry\0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDependencySpec {
    pub plugin_id: String,
    pub min_version: Version,
    pub max_version_exclusive: Version,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub required_abi_version: u32,
    pub description: String,
    pub dependencies: Vec<PluginDependencySpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn satisfies(self, min: Self, max_exclusive: Self) -> bool {
        self >= min && self < max_exclusive
    }
}

impl From<maruzzella_api::MzVersion> for Version {
    fn from(value: maruzzella_api::MzVersion) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
pub struct LoadedPlugin {
    path: PathBuf,
    descriptor: PluginDescriptor,
    vtable: &'static MzPluginVTable,
    _library: Library,
}

impl LoadedPlugin {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    pub fn vtable(&self) -> &'static MzPluginVTable {
        self.vtable
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginLoadError {
    LibraryOpen {
        path: PathBuf,
        message: String,
    },
    MissingEntryPoint {
        path: PathBuf,
        message: String,
    },
    NullVTable {
        path: PathBuf,
    },
    AbiMismatch {
        path: PathBuf,
        plugin_abi_version: u32,
    },
    DescriptorAbiMismatch {
        path: PathBuf,
        plugin_id: String,
        required_abi_version: u32,
    },
    InvalidUtf8 {
        path: PathBuf,
        field: &'static str,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginResolveError {
    DuplicatePluginId {
        plugin_id: String,
    },
    MissingRequiredDependency {
        plugin_id: String,
        dependency_id: String,
    },
    IncompatibleDependencyVersion {
        plugin_id: String,
        dependency_id: String,
        found_version: Version,
        min_version: Version,
        max_version_exclusive: Version,
    },
    DependencyCycle {
        plugin_ids: Vec<String>,
    },
}

type PluginEntryPoint = unsafe extern "C" fn() -> *const MzPluginVTable;

pub fn load_plugin(path: impl AsRef<Path>) -> Result<LoadedPlugin, PluginLoadError> {
    let path = path.as_ref().to_path_buf();
    let library = unsafe { Library::new(&path) }.map_err(|error| PluginLoadError::LibraryOpen {
        path: path.clone(),
        message: error.to_string(),
    })?;

    let entry: Symbol<'_, PluginEntryPoint> =
        unsafe { library.get(ENTRY_SYMBOL) }.map_err(|error| PluginLoadError::MissingEntryPoint {
            path: path.clone(),
            message: error.to_string(),
        })?;

    let vtable = unsafe { entry() };
    let Some(vtable) = (unsafe { vtable.as_ref() }) else {
        return Err(PluginLoadError::NullVTable { path });
    };

    if vtable.abi_version != MZ_ABI_VERSION_V1 {
        return Err(PluginLoadError::AbiMismatch {
            path,
            plugin_abi_version: vtable.abi_version,
        });
    }

    let descriptor_view = (vtable.descriptor)();
    let descriptor = descriptor_from_view(&path, descriptor_view)?;
    if descriptor.required_abi_version != MZ_ABI_VERSION_V1 {
        return Err(PluginLoadError::DescriptorAbiMismatch {
            path,
            plugin_id: descriptor.id,
            required_abi_version: descriptor.required_abi_version,
        });
    }

    Ok(LoadedPlugin {
        path,
        descriptor,
        vtable,
        _library: library,
    })
}

pub fn resolve_load_order<'a>(
    plugins: &'a [LoadedPlugin],
) -> Result<Vec<&'a LoadedPlugin>, PluginResolveError> {
    let mut by_id = HashMap::with_capacity(plugins.len());
    for plugin in plugins {
        if by_id
            .insert(plugin.descriptor.id.clone(), plugin)
            .is_some()
        {
            return Err(PluginResolveError::DuplicatePluginId {
                plugin_id: plugin.descriptor.id.clone(),
            });
        }
    }

    for plugin in plugins {
        for dependency in plugin
            .descriptor
            .dependencies
            .iter()
            .filter(|dependency| dependency.required)
        {
            let Some(target) = by_id.get(&dependency.plugin_id) else {
                return Err(PluginResolveError::MissingRequiredDependency {
                    plugin_id: plugin.descriptor.id.clone(),
                    dependency_id: dependency.plugin_id.clone(),
                });
            };
            let found_version = target.descriptor.version;
            if !found_version.satisfies(
                dependency.min_version,
                dependency.max_version_exclusive,
            ) {
                return Err(PluginResolveError::IncompatibleDependencyVersion {
                    plugin_id: plugin.descriptor.id.clone(),
                    dependency_id: dependency.plugin_id.clone(),
                    found_version,
                    min_version: dependency.min_version,
                    max_version_exclusive: dependency.max_version_exclusive,
                });
            }
        }
    }

    let mut indegree = HashMap::<String, usize>::with_capacity(plugins.len());
    let mut adjacency = HashMap::<String, Vec<String>>::with_capacity(plugins.len());
    for plugin in plugins {
        indegree.entry(plugin.descriptor.id.clone()).or_insert(0);
        adjacency.entry(plugin.descriptor.id.clone()).or_default();
    }

    for plugin in plugins {
        for dependency in plugin
            .descriptor
            .dependencies
            .iter()
            .filter(|dependency| dependency.required)
        {
            adjacency
                .entry(dependency.plugin_id.clone())
                .or_default()
                .push(plugin.descriptor.id.clone());
            *indegree.entry(plugin.descriptor.id.clone()).or_insert(0) += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(plugin_id, count)| (*count == 0).then_some(plugin_id.clone()))
        .collect::<Vec<_>>();
    ready.sort();
    let mut queue = VecDeque::from(ready);
    let mut ordered = Vec::with_capacity(plugins.len());

    while let Some(plugin_id) = queue.pop_front() {
        ordered.push(*by_id.get(&plugin_id).expect("plugin id disappeared"));
        if let Some(children) = adjacency.get(&plugin_id) {
            let mut newly_ready = Vec::new();
            for child in children {
                let count = indegree
                    .get_mut(child)
                    .expect("child plugin must exist in indegree");
                *count -= 1;
                if *count == 0 {
                    newly_ready.push(child.clone());
                }
            }
            newly_ready.sort();
            queue.extend(newly_ready);
        }
    }

    if ordered.len() != plugins.len() {
        let mut remaining = indegree
            .into_iter()
            .filter_map(|(plugin_id, count)| (count > 0).then_some(plugin_id))
            .collect::<Vec<_>>();
        remaining.sort();
        return Err(PluginResolveError::DependencyCycle {
            plugin_ids: remaining,
        });
    }

    Ok(ordered)
}

fn descriptor_from_view(
    path: &Path,
    descriptor: MzPluginDescriptorView,
) -> Result<PluginDescriptor, PluginLoadError> {
    let dependencies = dependency_slice(descriptor.dependencies_ptr, descriptor.dependencies_len)
        .iter()
        .map(|dependency| dependency_from_view(path, *dependency))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PluginDescriptor {
        id: decode_str(path, "id", descriptor.id)?,
        name: decode_str(path, "name", descriptor.name)?,
        version: descriptor.version.into(),
        required_abi_version: descriptor.required_abi_version,
        description: decode_str(path, "description", descriptor.description)?,
        dependencies,
    })
}

fn dependency_from_view(
    path: &Path,
    dependency: MzPluginDependency,
) -> Result<PluginDependencySpec, PluginLoadError> {
    Ok(PluginDependencySpec {
        plugin_id: decode_str(path, "dependency.plugin_id", dependency.plugin_id)?,
        min_version: dependency.min_version.into(),
        max_version_exclusive: dependency.max_version_exclusive.into(),
        required: dependency.required,
    })
}

fn decode_str(path: &Path, field: &'static str, value: MzStr) -> Result<String, PluginLoadError> {
    if value.len == 0 {
        return Ok(String::new());
    }
    if value.ptr.is_null() {
        return Err(PluginLoadError::InvalidUtf8 {
            path: path.to_path_buf(),
            field,
        });
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
    let value = str::from_utf8(bytes).map_err(|_| PluginLoadError::InvalidUtf8 {
        path: path.to_path_buf(),
        field,
    })?;
    Ok(value.to_string())
}

fn dependency_slice<'a>(
    ptr: *const MzPluginDependency,
    len: usize,
) -> &'a [MzPluginDependency] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn test_descriptor() -> MzPluginDescriptorView {
        MzPluginDescriptorView::empty()
    }

    extern "C" fn test_register(_: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn test_startup(_: *const maruzzella_api::MzHostApi) -> maruzzella_api::MzStatus {
        maruzzella_api::MzStatus::OK
    }

    extern "C" fn test_shutdown(_: *const maruzzella_api::MzHostApi) {}

    fn plugin(
        id: &str,
        version: Version,
        dependencies: Vec<PluginDependencySpec>,
    ) -> LoadedPlugin {
        LoadedPlugin {
            path: PathBuf::from(format!("{id}.so")),
            descriptor: PluginDescriptor {
                id: id.to_string(),
                name: id.to_string(),
                version,
                required_abi_version: MZ_ABI_VERSION_V1,
                description: String::new(),
                dependencies,
            },
            vtable: Box::leak(Box::new(MzPluginVTable {
                abi_version: MZ_ABI_VERSION_V1,
                descriptor: test_descriptor,
                register: test_register,
                startup: test_startup,
                shutdown: test_shutdown,
            })),
            _library: {
                #[cfg(unix)]
                {
                    libloading::os::unix::Library::this().into()
                }
                #[cfg(windows)]
                {
                    libloading::os::windows::Library::this()
                        .expect("current process library should be loadable")
                        .into()
                }
            },
        }
    }

    #[test]
    fn resolver_orders_dependencies_before_dependents() {
        let base = plugin("maruzzella.base", Version { major: 1, minor: 0, patch: 0 }, vec![]);
        let notes = plugin(
            "com.example.notes",
            Version { major: 1, minor: 0, patch: 0 },
            vec![PluginDependencySpec {
                plugin_id: "maruzzella.base".to_string(),
                min_version: Version { major: 1, minor: 0, patch: 0 },
                max_version_exclusive: Version { major: 2, minor: 0, patch: 0 },
                required: true,
            }],
        );

        let plugins = [notes, base];
        let ordered = resolve_load_order(&plugins).expect("dependencies should resolve");
        let ids = ordered
            .into_iter()
            .map(|plugin| plugin.descriptor.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["maruzzella.base", "com.example.notes"]);
    }

    #[test]
    fn resolver_rejects_missing_required_dependency() {
        let notes = plugin(
            "com.example.notes",
            Version { major: 1, minor: 0, patch: 0 },
            vec![PluginDependencySpec {
                plugin_id: "maruzzella.base".to_string(),
                min_version: Version { major: 1, minor: 0, patch: 0 },
                max_version_exclusive: Version { major: 2, minor: 0, patch: 0 },
                required: true,
            }],
        );

        let error = resolve_load_order(&[notes]).expect_err("dependency should be required");
        assert_eq!(
            error,
            PluginResolveError::MissingRequiredDependency {
                plugin_id: "com.example.notes".to_string(),
                dependency_id: "maruzzella.base".to_string(),
            }
        );
    }
}
