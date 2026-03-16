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
11. built-in `maruzzella.base` plugin for default shell contributions
12. built-in base-plugin-backed default shell slice replacing placeholder-first startup UI
13. host-side plugin configuration persistence keyed by plugin id
14. workbench tab open/focus/reuse APIs for plugins with instance identity and payload restore
15. typed contribution surfaces for About, settings entries, toolbar items, and startup tabs
16. typed host catalogs for commands, views, about sections, settings entries, diagnostics, and plugin runtime inventory
17. base-plugin-owned About, Plugins, and Settings shell pages
18. plugin settings entries that can open concrete plugin-owned settings views
19. example plugin exercising plugin-owned config-backed settings UI end to end

These pieces are enough to prove the basic architecture:

- downstream apps configure Maruzzella
- plugins are dynamic libraries
- plugins declare metadata and dependencies
- Maruzzella loads and activates plugins
- plugins can contribute commands and menus
- plugin commands can execute real code
- plugins can mount real GTK widgets into shell tabs
- the default visible shell experience can be base-plugin-owned
- shared shell surfaces can be explicit host contracts rather than ad hoc shell wiring

What is still explicitly not done:

- plugin configuration is still raw plugin-owned bytes with no schema/version/migration contract
- invalid or missing plugin config state is not yet modeled explicitly in the shell UI
- runtime services are still minimal beyond commands, views, catalogs, and config read/write
- packaging/discovery/docs for third-party plugin authors still need dedicated polish

## Guiding Direction

The intended architecture remains:

- `MaruzzellaConfig` and `ProductSpec` for product identity and shell defaults
- plugins for behavior, views, menus, commands, and contribution surfaces
- a built-in `maruzzella.base` plugin providing core shell facilities
- a strict ABI-safe boundary between host and plugin
- full Rust and GTK freedom inside plugins

## Completed Phases

### 1. Real Shell Slice

Outcome:

- the default app boots into a coherent base-plugin-backed shell rather than placeholder tabs

Delivered:

- neutral host scaffolding with visible default shell content moved into `maruzzella.base`
- base-plugin-backed startup views across workbench, side panel, and bottom panel
- base-plugin-owned visible commands and menus for core shell workflows

### 2. Contribution Surfaces

Outcome:

- shared shell areas now have explicit typed host contracts that multiple plugins can target

Delivered:

- stable typed surfaces for About, settings entries, toolbar items, and startup tabs
- typed host catalogs for commands, views, about sections, settings entries, diagnostics, and plugin inventory
- plugin workbench tab identity/open/focus/reuse APIs with instance-key and payload restore

### 3. Plugin Manager And Settings

Outcome:

- plugin inspection and settings are first-class base-plugin-owned shell pages

Delivered:

- base-plugin-owned Plugins page showing plugin identity, dependencies, diagnostics, logs, views, and settings entries
- base-plugin-owned Settings page aggregating plugin settings entries
- settings entries can open plugin-owned settings views through stable host APIs
- example plugin demonstrates config-backed plugin-owned settings UI

## Remaining Phases

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

The next implementation target should be **Plugin Configuration And Persistence**.

Reason:

- the real shell slice now exists and proves the product-first direction
- plugins and settings now have first-class base-plugin-owned shell pages
- settings surfaces can open plugin-owned settings views through stable host contracts
- the next gaps are richer config schemas, invalid-state handling, and migration/versioning hooks

## Notes On Sequencing

The order above is intentional:

- the shell must feel real before deeper platform work is worth standardizing
- contribution surfaces should be derived from working shell flows rather than invented in isolation
- plugin manager and settings become much more valuable once those shared surfaces are explicit

The exact slice boundaries may still shift, but changes should continue to preserve a clean path toward this roadmap.
