# Maruzzella

Maruzzella is an original GTK4 desktop shell prototype built in Rust.

It focuses on the shell itself rather than on any specific product domain: custom tabbed workbench areas, side panels, bottom panels, split layouts, lightweight command handling, and persisted workspace state.

## Current Status

Today the project is in an intermediate but coherent state:

- the GTK shell itself is working
- layout persistence is working
- the public crate API is usable by downstream apps
- the first plugin ABI/runtime slice is working for loading plugins, resolving dependencies, merging commands and menus, and dispatching plugin commands
- a built-in `maruzzella.base` plugin now provides core `About` and `Plugins` shell commands/menu entries
- plugin-backed GTK views can now be mounted into tabs
- the default app now boots into a coherent base-plugin-backed shell slice instead of placeholder-first `ProductSpec` text tabs
- plugin configuration persistence and plugin settings-summary surfaces are wired through the host
- plugin runtime services, host events, and discovery conventions are now available to downstreams

That means the plugin architecture is proven and the shell now exercises it in its default startup experience, but many shared shell contracts are still not formalized.

## What It Does

- renders a multi-pane desktop shell with left, right, bottom, and central workbench regions
- uses custom tab groups instead of `GtkNotebook`
- supports tab activation, reordering, and workbench split previews
- persists tab arrangement and pane sizes to a local JSON layout file
- exposes a small command surface for theme reload and command palette style actions
- ships with a built-in workspace slice that can be replaced or extended by downstream products

## Project Structure

- `src/app.rs`: application bootstrap, shell construction, pane persistence wiring
- `src/product.rs`: default branding, menus, toolbar actions, and starter layout
- `src/shell/`: shell UI components including top bar, tabbed panels, and custom workbench tabs
- `src/layout.rs`: load/save persisted shell state
- `src/spec.rs`: serializable shell, tab, menu, and workbench layout model
- `src/theme.rs`: runtime theme loading and token rendering
- `resources/style.css`: default theme template

## Run

Requirements:

- Rust toolchain
- GTK4 development libraries available on the system

Start the app with:

```bash
cargo run
```

Run the example app with:

```bash
cargo run --example notebook
```

Run the plugin view demo with:

```bash
cargo build -p example_plugin
cargo run --example plugin_view
```

## Public API

The intended integration surface is the crate root:

- `MaruzzellaConfig`: application id, persistence namespace, and product definition
- `ThemeSpec`: configurable theme tokens and optional external stylesheet template
- `MaruzzellaConfig::with_plugin_path(...)`: register a dynamic plugin library to load at startup
- `MaruzzellaConfig::with_plugin_dir(...)`: add a directory to plugin discovery
- `MaruzzellaConfig::without_default_plugin_discovery()`: opt out of the built-in discovery convention
- `MaruzzellaConfig::with_theme(...)`: swap typography, palette, sizing, or the whole stylesheet
- `default_plugin_discovery_dirs(...)`: inspect the built-in plugin discovery convention
- `discover_plugin_paths_in_dir(...)`: enumerate loadable plugin artifacts in a directory
- `run(config)`: launch a configured shell
- `build_application(config)`: build a GTK application without running it yet
- `ProductSpec` and related spec types: define branding, menus, toolbar actions, panels, and workbench layout

Minimal example:

```rust
use maruzzella::{default_product_spec, run, MaruzzellaConfig, ThemeSpec};

fn main() {
    let mut product = default_product_spec();
    product.branding.title = "My App".to_string();

    let config = MaruzzellaConfig::new("com.example.my-app")
        .with_persistence_id("my-app")
        .with_theme(ThemeSpec::default())
        .with_product(product);

    run(config);
}
```

Dynamic plugins can also be attached through config:

```rust
let config = MaruzzellaConfig::new("com.example.my-app")
    .with_plugin_path("plugins/libexample_plugin.so");
```

Discovery can also be directory-based. By default Maruzzella now scans:

- `$XDG_CONFIG_HOME/<persistence_id>/plugins`
- `./plugins`

You can add more directories or opt out of the built-in convention:

```rust
let config = MaruzzellaConfig::new("com.example.my-app")
    .with_plugin_dir("/opt/my-app/plugins")
    .without_default_plugin_discovery();
```

## Theming

The shell styling is no longer tied to a single baked-in CSS file.

- `ThemeSpec::default()` gives the bundled Maruzzella look
- `ThemeSpec` exposes typed palette, typography, and density tokens for common changes
- `ThemeDensity` also controls window defaults and panel minimum sizes
- `ThemeSpec::with_stylesheet_path(...)` points at an external GTK CSS template for full theme swapping
- `ThemeSpec::with_override(key, value)` injects extra `{{token}}` values for custom templates
- the bundled stylesheet is now expressed in semantic component tokens rather than anonymous literal placeholders

Minimal theme override:

```rust
use maruzzella::{MaruzzellaConfig, ThemeSpec};

let mut theme = ThemeSpec::default();
theme.typography.font_family = "\"IBM Plex Sans\", \"Cantarell\", sans-serif".to_string();
theme.typography.mono_font_family = "\"IBM Plex Mono\", monospace".to_string();
theme.palette.accent = "#d06b2f".to_string();
theme.density.radius_medium = 14;
theme.density.toolbar_height = 44;
theme.density.window_default_width = 1680;
theme.density.min_side_panel_width = 260;

let config = MaruzzellaConfig::new("com.example.my-app").with_theme(theme);
```

Full stylesheet swap:

```rust
use maruzzella::{MaruzzellaConfig, ThemeSpec};

let theme = ThemeSpec::default()
    .with_stylesheet_path("themes/sand/style.css")
    .with_override("hero_glow", "alpha(#d06b2f, 0.18)");

let config = MaruzzellaConfig::new("com.example.my-app").with_theme(theme);
```

The bundled `Reload Theme` command now reloads the active theme spec, including the external stylesheet file if one is configured.

## Persistence

By default, Maruzzella stores its layout at:

```text
$XDG_CONFIG_HOME/maruzzella/layout.json
```

If `XDG_CONFIG_HOME` is not set, it falls back to:

```text
$HOME/.config/maruzzella/layout.json
```

If you set `MaruzzellaConfig::with_persistence_id(...)`, the directory name changes accordingly.

Deleting the layout file resets the shell to the default layout supplied in the config's `ProductSpec`.

## Extending It

The easiest way to evolve the shell is to change the default `ProductSpec` in `src/product.rs`:

- rename branding and window text
- add commands, menus, and toolbar items
- replace placeholder tabs with real views
- reshape the central workbench tree with horizontal or vertical splits

The shell model in `src/spec.rs` is serializable, so layouts can be generated or persisted without coupling them to widget code.

## Plugin Author Workflow

The repeatable plugin author path is now:

1. Create a `cdylib` crate that depends on `maruzzella_sdk`.
2. Export the plugin with `export_plugin!(YourPlugin)`.
3. Build the library for the current platform.
4. Either point the host at the exact library with `with_plugin_path(...)` or place it in one of the discovery directories.

The sample plugin in [plugins/example_plugin](/home/lelloman/lelloprojects/maruzzella/plugins/example_plugin) demonstrates:

- command, menu, toolbar, settings, and view contributions
- host-owned config persistence
- service registration
- host event subscription

See [docs/plugin-author-workflow.md](/home/lelloman/lelloprojects/maruzzella/docs/plugin-author-workflow.md) for a concrete workflow.

## Versioning Policy

`maruzzella_api` now follows a simple policy:

- additive changes that preserve `MZ_ABI_VERSION_V1` keep the ABI version stable
- any breaking C-ABI layout or semantic incompatibility requires a new ABI constant and corresponding host/plugin upgrade
- `maruzzella_sdk` is expected to track the API crate closely and should be upgraded together in downstream plugin work

## Design Notes

The first plugin ABI draft lives in [docs/plugin-abi-rfc.md](/home/lelloman/lelloprojects/maruzzella/docs/plugin-abi-rfc.md).

The workspace now also contains:

- `maruzzella_api`: ABI-safe plugin boundary types
- `maruzzella_sdk`: ergonomic Rust helpers and export macro for plugin authors
- `maruzzella.base`: built-in plugin providing core shell commands and menu contributions
- `plugins/example_plugin`: sample `cdylib` plugin using the SDK

On the host side, `maruzzella` now exposes the first loading and activation primitives:

- `load_plugin(path)`: open a dynamic library and decode its descriptor
- `resolve_load_order(&plugins)`: dependency-aware ordering
- `PluginRuntime::activate(plugins)`: invoke plugin `register` and `startup` against a host API and collect contributions

Plugin commands are now executable, not just declarative metadata: a plugin can register a command together with an ABI-safe handler function, and Maruzzella will dispatch GTK menu actions back into that plugin.

What is still rough:

- some contribution surfaces are still intentionally small
- packaging beyond raw platform libraries is not standardized
- the SDK can still be tightened further as more third-party plugins appear
