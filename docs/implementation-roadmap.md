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
10. plugin-backed tab views
11. built-in `maruzzella.base` plugin for `About` and `Plugins` shell contributions
12. built-in base-plugin-backed default shell slice replacing placeholder-first startup UI
13. host-side plugin configuration persistence and settings-summary surfaces
14. in-app plugin manager dialog with dependency, diagnostics, settings, and runtime-log visibility

These pieces are enough to prove the basic architecture:

- downstream apps configure Maruzzella
- plugins are dynamic libraries
- plugins declare metadata and dependencies
- Maruzzella loads and activates plugins
- plugins can contribute commands and menus
- plugin commands can execute real code
- plugins can mount real GTK widgets into shell tabs
- core shell behavior can begin moving behind the plugin runtime

What is still explicitly not done:

- contribution surfaces are still limited and partially stringly typed
- the plugin manager is a useful dialog, but not yet a full first-class shell page
- plugin configuration exists as host storage and settings summaries, but not as a rich settings UI contract

## Guiding Direction

The intended architecture remains:

- `MaruzzellaConfig` and `ProductSpec` for product identity and shell defaults
- plugins for behavior, views, menus, commands, and contribution surfaces
- a built-in `maruzzella.base` plugin providing core shell facilities
- a strict ABI-safe boundary between host and plugin
- full Rust and GTK freedom inside plugins

## Remaining Phases

### 1. Real Shell Slice

Goal:

- make the default app feel like a credible downstream shell instead of a shell prototype

Work:

- keep the built-in base plugin as the reference shell provider
- ensure the default startup layout lands in real base-plugin-backed views across the workbench and side panels
- use this slice to prove shell hierarchy, plugin manager visibility, and shell contribution contracts together
- avoid reintroducing placeholder-first `ProductSpec` content in prime UI areas

Exit condition:

- a downstream app can launch Maruzzella and immediately see a coherent shell workflow without editing defaults

### 2. Contribution Surfaces

Goal:

- replace ad hoc contribution wiring with stable, explicit surfaces

Work:

- formalize shared surface ids
- define structured contracts for shell-level surfaces
- expand beyond the current about/settings summaries into:
  - panel/view contribution categories
  - settings page contracts
  - richer menu and toolbar contribution surfaces
  - status or diagnostics surfaces if they prove useful
- move common contracts into `maruzzella_api`

Exit condition:

- multiple plugins can contribute to shared shell areas through stable contracts

### 3. Plugin Manager And Settings

Goal:

- turn the existing plugin manager and settings summaries into a proper shell experience

Work:

- promote plugin management from dialog-only UI into a first-class shell page or equivalent richer surface
- list plugin id, version, description, dependency state, and activation/runtime errors
- render plugin-provided settings/config surfaces through the shared surface model
- keep About and Plugins flows aligned with the same host-owned contracts

Exit condition:

- users can inspect installed and active plugins from inside Maruzzella without relying on placeholder host UI

### 4. Plugin Configuration And Persistence

Goal:

- support plugin-owned persistent configuration hosted by Maruzzella

Work:

- build on top of the existing host-side config storage keyed by plugin id
- expose richer plugin read/write config APIs and stable settings UI contracts
- make invalid/missing config states visible in the shell UI
- reserve version-aware config migration hooks

Exit condition:

- plugins can store and retrieve stable configuration through the host and expose it cleanly in-app

### 5. Richer Runtime Services

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

### 6. Polish And Packaging

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

The next implementation target should be **Plugin Manager And Settings**.

Reason:

- the real shell slice now exists and proves the product-first direction
- commands, menus, plugin-backed views, typed shell catalogs, and plugin config persistence are all live
- shared settings and diagnostics contracts now exist alongside menu/toolbar/startup surfaces
- the next gains come from turning those contracts into a richer plugin manager and settings experience

## Notes On Sequencing

The order above is intentional:

- the shell must feel real before deeper platform work is worth standardizing
- contribution surfaces should be derived from working shell flows rather than invented in isolation
- plugin manager and settings become much more valuable once those shared surfaces are explicit

The exact slice boundaries may still shift, but changes should continue to preserve a clean path toward this roadmap.
