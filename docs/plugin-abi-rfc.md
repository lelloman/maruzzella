# Plugin ABI RFC v1

## Goal

Maruzzella should support dynamic plugins from the beginning without giving up:

- rich Rust implementations inside plugins
- custom GTK widgets created by plugins
- plugin dependency resolution
- stable host/plugin loading semantics

At the same time, Maruzzella must not rely on the unstabilized Rust ABI across dynamic library boundaries.

This RFC defines the first plugin boundary for Maruzzella.

## Non-Goals

This RFC does not define:

- final end-user plugin packaging or installation UX
- a networked or sandboxed plugin runtime
- plugin unloading
- hot reloading
- a finalized configuration schema
- every possible shell contribution surface

## Core Principles

1. Plugins are native dynamic libraries loaded at runtime.
2. Plugin internals may use full Rust and gtk-rs.
3. The host/plugin boundary must be ABI-safe.
4. Plugins do not arbitrarily mutate shell UI.
5. Plugins contribute structure and behavior through host-owned registration APIs.
6. GTK objects may cross the boundary only through GObject/GTK-compatible pointer forms.
7. Active plugins are never unloaded during process lifetime.

## Layering

The plugin architecture is split into four layers:

- `maruzzella`
  The host application, plugin loader, registry, runtime, and shell renderer.
- `maruzzella_api`
  ABI-safe types, C-compatible entrypoints, host callback tables, plugin descriptors, and shared contribution contracts.
- `maruzzella_sdk`
  Ergonomic Rust wrappers for plugin authors that hide most ABI details.
- plugin crates
  Independent dynamic libraries compiled outside Maruzzella and loaded at runtime.

The SDK is the ergonomic layer. The API crate is the binary contract.

## Startup Model

At startup, Maruzzella should:

1. Build a `ProductConfig`.
2. Load the built-in base plugin.
3. Discover user-provided plugin libraries.
4. Resolve plugin descriptors and dependency graph.
5. Reject or disable plugins with unsatisfied hard dependencies.
6. Register contributions from resolved plugins.
7. Build menus, actions, settings surfaces, and shell views from merged contributions.
8. Create plugin-provided widgets on demand.

## ProductConfig Scope

`ProductConfig` should remain intentionally small. It should contain:

- application id
- persistence namespace
- branding
- basic shell defaults
- plugin search paths or plugin manifests

It should not become the place where downstream products directly assemble full shell behavior.

## Plugin Model

Each plugin has:

- an id
- a semantic version
- an API compatibility version
- a dependency list
- optional contribution declarations
- optional runtime hooks

Each plugin should be uniquely identified by a stable string id, for example:

- `maruzzella.base`
- `com.example.notes`
- `org.example.git`

## Exported Entry Point

Each plugin dynamic library exports one known symbol:

```rust
extern "C" fn maruzzella_plugin_entry() -> *const MzPluginVTable
```

The vtable is the root ABI object for the plugin.

## Plugin VTable

The exact naming may change, but v1 should contain at least:

- `abi_version`
- `descriptor`
- `register`
- `startup`
- `shutdown`

Conceptually:

```rust
#[repr(C)]
pub struct MzPluginVTable {
    pub abi_version: u32,
    pub descriptor: extern "C" fn() -> MzPluginDescriptorView,
    pub register: extern "C" fn(host: *const MzHostApi) -> MzStatus,
    pub startup: extern "C" fn(host: *const MzHostApi) -> MzStatus,
    pub shutdown: extern "C" fn(host: *const MzHostApi),
}
```

`register` is for declaring contributions and handlers.

`startup` is for imperative initialization after registration succeeds.

`shutdown` is best-effort cleanup only. It must not imply plugin unloading.

## Descriptor

The plugin descriptor must include:

- plugin id
- human-readable name
- semantic version
- required Maruzzella ABI version
- optional description
- dependency list

Conceptually:

```rust
#[repr(C)]
pub struct MzPluginDescriptorView {
    pub id: MzStr,
    pub name: MzStr,
    pub version: MzVersion,
    pub required_abi_version: u32,
    pub description: MzStr,
    pub dependencies_ptr: *const MzPluginDependency,
    pub dependencies_len: usize,
}
```

## Dependencies

Plugin dependencies are runtime dependencies, not automatic Rust code dependencies.

Each dependency should declare:

- target plugin id
- minimum version
- maximum version or compatible major range later
- required vs optional

Conceptually:

```rust
#[repr(C)]
pub struct MzPluginDependency {
    pub plugin_id: MzStr,
    pub min_version: MzVersion,
    pub max_version_exclusive: MzVersion,
    pub required: bool,
}
```

Resolution rules for v1:

- if a required dependency is missing, the plugin is not activated
- if a required dependency version is incompatible, the plugin is not activated
- optional dependencies may be ignored for v1
- dependency cycles are an error

## ABI Rules

Across the dynamic library boundary, Maruzzella must not expose ordinary Rust ABI types as the contract.

Forbidden as boundary types:

- `String`
- `&str`
- `Vec<T>`
- plain Rust trait objects
- `Box<T>` with cross-boundary ownership
- normal Rust closures
- ordinary Rust enums without explicit ABI design
- GTK Rust wrapper objects like `gtk::Widget`

Allowed boundary forms:

- primitive integers and booleans
- `#[repr(C)]` structs
- explicitly ABI-safe tagged enums
- pointer + length views
- opaque handles
- `extern "C"` function pointers
- UTF-8 string views
- byte slices
- GObject or GTK instance pointers

## Shared ABI Helper Types

The API crate should define a small set of shared ABI-safe helpers:

- `MzStr`
  UTF-8 bytes as pointer + length
- `MzBytes`
  opaque byte payload
- `MzVersion`
  semantic version components
- `MzStatus`
  success/error status code
- `MzHandle`
  opaque numeric or pointer handle

Example:

```rust
#[repr(C)]
pub struct MzStr {
    pub ptr: *const u8,
    pub len: usize,
}
```

The SDK should wrap these into ergonomic Rust types on the plugin side.

## Host API

The host exposes a callback table to plugins during registration and runtime.

The host API should cover only controlled operations. It should not expose arbitrary shell internals.

The host must provide registration functions for:

- commands
- menu items
- toolbar items
- settings pages
- dialogs or modal entries
- contribution surfaces
- view factories

The host API should also provide runtime services for:

- logging
- querying plugin metadata
- config read/write
- command dispatch
- looking up registered surfaces
- optional service discovery later

Conceptually:

```rust
#[repr(C)]
pub struct MzHostApi {
    pub abi_version: u32,
    pub log: extern "C" fn(level: u32, message: MzStr),
    pub register_command: extern "C" fn(command: *const MzCommandSpec) -> MzStatus,
    pub register_menu_item: extern "C" fn(item: *const MzMenuItemSpec) -> MzStatus,
    pub register_surface_contribution: extern "C" fn(contribution: *const MzSurfaceContribution) -> MzStatus,
    pub register_view_factory: extern "C" fn(factory: *const MzViewFactorySpec) -> MzStatus,
    pub dispatch_command: extern "C" fn(command_id: MzStr, payload: MzBytes) -> MzStatus,
}
```

The final host API will likely be split into separate registration and runtime tables, but this is enough for v1 design.

## Declarative Contributions

Plugins must be able to contribute declarative structure to the shell, including:

- commands
- menus
- toolbar items
- settings pages
- dialogs
- view descriptors
- shell surfaces
- contributions to existing surfaces

Examples of host-owned surfaces:

- `maruzzella.menu.file.items`
- `maruzzella.menu.help.items`
- `maruzzella.about.sections`
- `maruzzella.plugins.settings_pages`

The base plugin should define the first shared shell-level contribution surfaces.

## Imperative Behavior

Plugins must also support arbitrary code execution through controlled hooks.

V1 hooks should include:

- registration
- startup
- shutdown
- command invocation
- widget creation

This allows plugins to:

- initialize state
- spawn async work later
- handle actions
- create dynamic UI
- react to shell requests

Imperative behavior must always flow through host-defined entrypoints and contracts.

## Commands

Commands are globally identified by string ids.

A plugin may register:

- command metadata
- a command handler entrypoint

The host owns command dispatch. Plugins do not directly wire menu callbacks into shell widgets.

Command payloads for v1 should be byte payloads or empty payloads. The SDK may offer typed serialization helpers on top.

## Menus

The root menu should begin empty at the core host level.

Menus should be assembled from plugin contributions.

The built-in base plugin contributes at least:

- a `File` menu
- a `Plugins` entry under `File`
- a `Help` menu or equivalent
- an `About` entry

Menu items should reference command ids, not direct callbacks.

## Base Plugin

`maruzzella.base` is the first real plugin and must be loaded by default.

Its responsibilities:

- register core shell commands
- define root shell contribution surfaces
- provide the plugins management modal
- provide the about modal
- contribute base menu items
- host plugin configuration UI surfaces

The base plugin should not hardcode downstream product metadata. It should read product branding from host-provided product context.

Other plugins should be able to depend on `maruzzella.base` and contribute, for example, additional about sections.

## GTK Widget Factories

Plugins must be able to create custom GTK widgets.

This is allowed because GTK and GObject already operate on a stable C ABI.

The important rule is that the ABI boundary uses GObject/GTK-compatible pointers, not ordinary Rust wrapper types.

For example, a plugin may register a view factory identified by a string id. When the host needs that view, it calls into the plugin and receives a widget pointer.

Conceptually:

```rust
#[repr(C)]
pub struct MzViewFactorySpec {
    pub plugin_id: MzStr,
    pub view_id: MzStr,
    pub create: extern "C" fn(host: *const MzHostApi, request: *const MzViewRequest) -> *mut gtk_sys::GtkWidget,
}
```

The final API crate should not depend directly on a high-level gtk-rs crate for this ABI. It should use raw GTK/GObject pointer forms or compatible wrappers.

## Widget Ownership And Lifetime

V1 ownership rules:

- the plugin creates the widget instance
- the plugin transfers ownership to the host as a GTK object reference
- the host becomes responsible for normal GTK lifetime management after adoption
- the plugin library must remain loaded for the life of any object whose implementation lives in that library
- active plugins are therefore never unloaded

Any plugin that registers GTypes, subclasses GTK widgets, installs signals, or returns widget instances must be considered permanently resident.

## Structured Contribution Contracts

Some contribution surfaces need structured payloads, such as:

- about sections
- settings pages
- diagnostics providers

These contracts must not be invented ad hoc inside a plugin if other plugins are expected to consume them.

They must live in a shared contract location:

- `maruzzella_api` for generic shell-level contracts
- dedicated shared API crates for domain-specific ecosystems

This allows:

- compile-time agreement on the contract
- runtime dependency on the provider plugin

Example:

- `maruzzella.base` hosts `maruzzella.about.sections`
- the shape of an about section is defined in `maruzzella_api`
- another plugin depends at runtime on `maruzzella.base`
- that plugin contributes an about section using the shared contract type

If a structured contract becomes too unstable or too rich for the v1 ABI, it should cross the boundary as serialized bytes and be decoded by the host-side SDK layer.

## Error Handling

Plugin-facing ABI calls should return explicit status codes, not panic across the boundary.

Rules:

- panics must not unwind across the host/plugin ABI boundary
- plugin-side SDK should catch panics and translate them into error statuses where possible
- malformed contributions are rejected by the host with diagnostics
- incompatible ABI version prevents plugin activation

## Discovery And Packaging

V1 discovery may be simple:

- explicit plugin library paths in config
- a plugin directory scanned at startup

Packaging format can remain unspecified for now. The key requirement is that the host receives a filesystem path to a dynamic library compatible with the current platform.

## Platform Artifacts

Plugin libraries will be platform-native artifacts, for example:

- `.so` on Linux
- `.dylib` on macOS
- `.dll` on Windows

The API and SDK should avoid platform-specific assumptions outside library loading details.

## Security

V1 plugins are trusted native code.

This implies:

- full process access
- no isolation
- no sandboxing

That is acceptable for v1, but it must be stated clearly in documentation.

## Implementation Order

Recommended implementation order:

1. Create `maruzzella_api` with ABI-safe primitives and plugin descriptor types.
2. Create `maruzzella_sdk` with safe wrappers around descriptor/export boilerplate.
3. Add loader support in `maruzzella` using `libloading`.
4. Implement plugin descriptor loading and dependency resolution.
5. Implement `maruzzella.base` as the first plugin.
6. Implement command and menu contribution registration.
7. Implement widget factory registration.
8. Add one external sample plugin outside the core crate tree.

## Open Questions

- Should structured payloads default to ABI-safe structs or serialized bytes?
- Should view factories be synchronous only in v1?
- How much of plugin configuration belongs in the base plugin vs host core?
- Do we want plugin enable/disable state persisted by plugin id and version?
- Do we need plugin capability flags in addition to dependencies?
- Should the host expose a generic service registry in v1 or later?

## Recommended Decision

Proceed with dynamic plugins from day one, but only with:

- a strict ABI-safe boundary
- a dedicated API crate
- an SDK for ergonomic Rust authoring
- a built-in base plugin that proves the contribution model
- no plugin unloading

This preserves the original product direction while avoiding the fragility of pretending that ordinary Rust dylib boundaries are stable enough for a plugin platform.
