# Implementation Roadmap

This document captures the current high-level implementation plan for Maruzzella.

It is not a strict waterfall plan. The project is being built in stable vertical slices, but the overall direction should remain explicit.

## Current Status

Implemented so far:

1. standalone public shell API for downstream apps
2. plugin ABI RFC
3. `maruzzella_api` crate
4. `maruzzella_sdk` crate
5. sample dynamic plugin crate
6. host-side dynamic plugin loader
7. host-side plugin activation runtime
8. plugin runtime integration into app startup
9. plugin command dispatch from GTK actions into plugin handlers

These pieces are enough to prove the basic architecture:

- downstream apps configure Maruzzella
- plugins are dynamic libraries
- plugins declare metadata and dependencies
- Maruzzella loads and activates plugins
- plugins can contribute commands and menus
- plugin commands can execute real code

What is still explicitly not done:

- the shell renderer does not yet instantiate plugin-provided GTK widgets
- default UI content is still mostly placeholder `ProductSpec` tabs
- there is no built-in `maruzzella.base` plugin yet
- plugin manager UI and plugin configuration storage do not exist yet

## Guiding Direction

The intended architecture remains:

- `MaruzzellaConfig` and `ProductSpec` for product identity and shell defaults
- plugins for behavior, views, menus, commands, and contribution surfaces
- a built-in `maruzzella.base` plugin providing core shell facilities
- a strict ABI-safe boundary between host and plugin
- full Rust and GTK freedom inside plugins

## Remaining Phases

### 1. Plugin Views

Goal:

- let plugins provide real widgets inside Maruzzella panels and workbench tabs

Work:

- extend the shell spec so a tab can reference a plugin-owned view id
- wire `PluginRuntime` view factories into shell tab construction
- let shell panels render plugin widgets instead of only placeholder text content
- define ownership and error behavior for failed widget creation
- update the sample plugin to prove the full path with one real view

Exit condition:

- a plugin can register a view factory and a downstream app can place that view in the shell

### 2. Base Plugin

Goal:

- move core shell behavior out of hardcoded host logic and into `maruzzella.base`

Work:

- define `maruzzella.base`
- make top-level menu roots effectively plugin-driven
- move `Plugins` and `About` commands/menu items into the base plugin
- move shell contribution surfaces into the base plugin or shared API as appropriate

Exit condition:

- Maruzzella’s own core shell functionality is demonstrated through the plugin system

### 3. Contribution Surfaces

Goal:

- replace ad hoc contribution wiring with stable, explicit surfaces

Work:

- formalize shared surface ids
- define structured contracts for shell-level surfaces
- start with:
  - `maruzzella.about.sections`
  - plugin settings/config pages
  - menu contribution surfaces
- move common contracts into `maruzzella_api`

Exit condition:

- multiple plugins can contribute to shared shell areas through stable contracts

### 4. Plugin Manager UI

Goal:

- expose loaded plugin state inside the shell

Work:

- plugins modal/page
- list plugin id, version, description, dependency state
- show activation/runtime errors
- show plugin-provided settings/config surfaces

Exit condition:

- users can inspect installed and active plugins from inside Maruzzella

### 5. Plugin Configuration And Persistence

Goal:

- support plugin-owned persistent configuration hosted by Maruzzella

Work:

- config storage keyed by plugin id
- plugin read/write config APIs
- settings UI integration
- version-aware config migration later if needed

Exit condition:

- plugins can store and retrieve stable configuration through the host

### 6. Richer Runtime Services

Goal:

- make the plugin host practical for more than static menus and basic commands

Work:

- command payload support beyond empty payloads
- better runtime diagnostics
- optional service registry
- host events or lifecycle subscriptions
- structured error surfacing to UI

Exit condition:

- plugin interactions are rich enough for real product integrations

### 7. Polish And Packaging

Goal:

- make the plugin system usable by downstream products and third-party plugin authors

Work:

- plugin discovery conventions
- platform-specific loading details
- build/documentation examples
- versioning policy for `maruzzella_api`
- SDK ergonomics cleanup

Exit condition:

- plugin author workflow is documented and repeatable

## Immediate Next Step

The next implementation target should be **Plugin Views**.

Reason:

- commands are now executable
- menus are now live
- the next missing piece is real plugin-provided UI
- without views, plugins still cannot meaningfully inhabit the shell
- once views exist, the base plugin and plugin manager become worth building

## Notes On Sequencing

The order above is intentional:

- views must come before plugin-hosted shell experiences feel real
- base plugin should come after the runtime is proven enough to host Maruzzella itself
- plugin manager and settings become much more valuable once views and contribution surfaces exist

The exact slice boundaries may still shift, but changes should continue to preserve a clean path toward this roadmap.
