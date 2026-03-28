# Appearance API

Maruzzella now treats shell styling as a semantic API instead of a downstream CSS exercise.

The intended model is:

- downstream apps define or override named appearances in `ThemeSpec`
- shell specs reference those appearances by stable ids
- the shell maps those ids to GTK/CSS internally
- raw stylesheet overrides remain available as an escape hatch

## Two Layers

There are two cooperating layers.

### 1. Appearance Registry In `ThemeSpec`

`ThemeSpec` owns named appearance recipes for:

- surfaces
- buttons
- text
- inputs
- tab strips

The main builder methods are:

- `with_surface_appearance(id, SurfaceAppearance)`
- `with_button_appearance(id, ButtonAppearance)`
- `with_text_appearance(id, TextAppearance)`
- `with_input_appearance(id, InputAppearance)`
- `with_tab_strip_appearance(id, TabStripAppearance)`

This is where a downstream app defines what `primary`, `secondary`, `danger`, or `title` means for that product.

### 2. Appearance References In Shell Specs

The shell spec types reference those appearances by id:

- `ShellSpec`
- `TabGroupSpec`
- `TabSpec`
- `ToolbarItemSpec`

This is where a downstream app says which role a widget should use.

Example:

```rust
use maruzzella::{
    default_product_spec, ButtonAppearance, ButtonStyle, MaruzzellaConfig, SurfaceAppearance,
    SurfaceLevel, TextRole, ThemeSpec, Tone,
};

let theme = ThemeSpec::default()
    .with_surface_appearance(
        "secondary",
        SurfaceAppearance::new(Tone::Secondary, SurfaceLevel::Raised, TextRole::Body),
    )
    .with_button_appearance(
        "primary",
        ButtonAppearance::new(Tone::Accent, ButtonStyle::Solid, TextRole::BodyStrong),
    );

let mut product = default_product_spec();
product.layout.left_panel = product
    .layout
    .left_panel
    .clone()
    .with_panel_appearance("secondary")
    .with_panel_header_appearance("secondary")
    .with_tab_strip_appearance("utility");

let config = MaruzzellaConfig::new("com.example.my-app")
    .with_theme(theme)
    .with_product(product);
```

## Built-In Appearance Ids

These ids are available by default and may be overridden by downstream apps by reusing the same id.

### Surface ids

- `app-shell`
- `topbar`
- `menu`
- `toolbar`
- `status`
- `primary`
- `secondary`
- `tertiary`
- `workbench`
- `console`
- `inspector`

### Button ids

- `primary`
- `secondary`
- `ghost`
- `toolbar`
- `icon`
- `danger`

### Text ids

- `title`
- `subtitle`
- `body`
- `body-strong`
- `meta`
- `section-label`
- `tab-label`
- `code`

### Input ids

- `search`
- `command`
- `field`

### Tab strip ids

- `editor`
- `utility`
- `console`

## Which Specs Control What

### `ShellSpec`

Top-level shell appearance ids:

- `app_appearance_id`
- `topbar_appearance_id`
- `menu_appearance_id`
- `toolbar_appearance_id`
- `search_input_appearance_id`
- `status_appearance_id`
- `button_appearance_id`
- `text_appearance_id`

### `TabGroupSpec`

Per-group appearance ids:

- `panel_appearance_id`
- `panel_header_appearance_id`
- `tab_strip_appearance_id`
- `text_appearance_id`

Convenience builders:

- `with_panel_appearance(...)`
- `with_panel_header_appearance(...)`
- `with_tab_strip_appearance(...)`
- `with_text_appearance(...)`

### `TabSpec`

Per-tab content appearance:

- `text_appearance_id`
- `with_text_appearance(...)`

### `ToolbarItemSpec`

Per-item button appearance:

- `appearance_id`

## Typography

Typography is now standardized through text roles rather than scattered widget classes.

Common roles:

- `title`: main emphasis
- `subtitle`: secondary heading
- `body`: standard UI/body copy
- `body-strong`: emphasized UI/body copy
- `meta`: low-emphasis supporting text
- `section-label`: uppercase shell labels
- `tab-label`: tab text
- `code`: monospace content

Downstreams should prefer assigning a text role or overriding a built-in text id instead of changing label CSS directly.

## Buttons

Buttons are standardized through semantic button ids and shell-owned interaction states.

Common roles:

- `primary`: strongest action
- `secondary`: normal structured action
- `ghost`: low-chrome action
- `toolbar`: toolbar action
- `icon`: icon-only action
- `danger`: destructive action

Downstreams should set `ToolbarItemSpec.appearance_id` or override the corresponding button appearance in `ThemeSpec`.

## Escape Hatch

`ThemeSpec::with_stylesheet_path(...)` and `ThemeSpec::with_override(...)` still work.

Use them when:

- a product needs a shell treatment not yet expressible with the semantic API
- a legacy downstream app is still migrating
- the product wants a near-total visual rewrite

They are no longer the preferred downstream customization path.
