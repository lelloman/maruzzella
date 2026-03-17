# Plugin Author Workflow

This is the intended downstream workflow for a Maruzzella plugin author.

## 1. Create A `cdylib`

Use a normal Rust library crate and expose a dynamic library:

```toml
[lib]
crate-type = ["cdylib"]
```

Depend on `maruzzella_sdk` and implement `Plugin`.

## 2. Export The Plugin

Use the SDK export macro:

```rust
use maruzzella_sdk::{export_plugin, Plugin};

struct MyPlugin;

impl Plugin for MyPlugin {
    // descriptor/register/startup/shutdown
}

export_plugin!(MyPlugin);
```

## 3. Build The Library

Build for the current platform:

```bash
cargo build -p example_plugin
```

The platform artifact will be:

- Linux: `.so`
- macOS: `.dylib`
- Windows: `.dll`

Typical debug artifact names are:

- Linux: `target/debug/libyour_plugin.so`
- macOS: `target/debug/libyour_plugin.dylib`
- Windows: `target/debug/your_plugin.dll`

## 4. Load It In Maruzzella

You have two supported host-side options.

Explicit path:

```rust
let config = MaruzzellaConfig::new("com.example.my-app")
    .with_plugin_path("target/debug/libexample_plugin.so");
```

Discovery directory:

- place the built library in `$XDG_CONFIG_HOME/<persistence_id>/plugins`, or
- place it in `./plugins`, or
- add a custom directory with `with_plugin_dir(...)`

Default discovery can be disabled with `without_default_plugin_discovery()`.

## Packaging Notes

For now Maruzzella expects raw platform-native dynamic libraries, not packaged bundles.

Recommended downstream conventions:

- ship plugins as separate platform artifacts
- copy them into a per-app plugin directory at install time
- keep third-party plugins out of the app binary directory unless the product explicitly wants that layout

Recommended install targets:

- Linux: `$XDG_CONFIG_HOME/<persistence_id>/plugins` for local development, or an app-managed plugin dir for packaged apps
- macOS: an app-managed plugin dir plus `with_plugin_dir(...)` is the cleanest current path
- Windows: an app-managed plugin dir plus `with_plugin_dir(...)` is the cleanest current path

The host-side discovery logic only filters by platform library extension right now. It does not impose signing, manifests, or packaging metadata.

## 5. Shared Contracts

Use `maruzzella_api` types for structured host/plugin contracts such as:

- settings pages
- about sections
- startup tabs
- toolbar items
- services
- host events

That keeps plugins aligned with the host without inventing ad hoc JSON formats for shared surfaces.

## 6. Versioning Expectations

- `MZ_ABI_VERSION_V1` means host and plugin agree on the current ABI layout
- additive host/plugin API growth can stay within v1
- breaking ABI changes require a new ABI constant and a coordinated upgrade

## 7. Reference Implementation

The sample plugin in [plugins/example_plugin](/home/lelloman/lelloprojects/maruzzella/plugins/example_plugin) is the reference implementation for plugin authors in this repository.
