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
- plugin-backed GTK views can now be mounted into tabs, but most default shell content is still placeholder `ProductSpec` content

That means the plugin architecture is proven and plugins can now inhabit the shell UI, but most shared shell contracts are still not formalized.

## What It Does

- renders a multi-pane desktop shell with left, right, bottom, and central workbench regions
- uses custom tab groups instead of `GtkNotebook`
- supports tab activation, reordering, and workbench split previews
- persists tab arrangement and pane sizes to a local JSON layout file
- exposes a small command surface for theme reload and command palette style actions
- ships with neutral placeholder content so the shell can be adapted to other tools

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
- `MaruzzellaConfig::with_theme(...)`: swap typography, palette, sizing, or the whole stylesheet
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

What is not done yet:

- plugin manager UI, plugin settings surfaces, and plugin-owned persistence are still planned work

Plugin views are now wired, so the next major targets in [docs/implementation-roadmap.md](/home/lelloman/lelloprojects/maruzzella/docs/implementation-roadmap.md) are deeper contribution surfaces, richer plugin management UI, and plugin-owned configuration/persistence.
