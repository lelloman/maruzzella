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
20. schema-aware plugin configuration persistence with config-state surfacing
21. payload-aware command dispatch through menus and toolbar items
22. runtime diagnostics, service registry, and host event subscriptions
23. plugin discovery conventions for explicit paths and discovery directories
24. plugin author workflow and packaging/versioning documentation
25. SDK ergonomics helpers for typed JSON payload, config, and service handling
26. built-in base-plugin editor tabs with untitled buffers, file-backed documents, native open/save-as flows, and host-backed draft restore
27. semantic shell appearance API for panels, tab strips, buttons, inputs, and typography roles

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

- plugin packaging is still raw platform-native library distribution rather than a richer package format
- discovery conventions exist, but signed/distributed third-party plugin installation is not standardized
- the SDK is cleaner, but more convenience wrappers may still be worth adding as third-party usage grows
- the roadmap itself no longer needs to drive the next slice; follow-up work should now be driven by stabilization and downstream adoption needs
- downstream products no longer need to treat Maruzzella styling as ad hoc CSS override work for common shell surfaces

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

### 4. Plugin Configuration And Persistence

Outcome:

- plugin-owned persistent configuration is now schema-aware, host-backed, and visible in shell settings UI

Delivered:

- host-owned config records with optional schema version metadata
- richer config read/write APIs in the ABI and SDK
- settings-page config contracts with missing/ready/migration-required/invalid state summaries
- reserved migration hook identifiers for future config migrations

### 5. Richer Runtime Services

Outcome:

- the plugin host now supports real payload-carrying interactions and shared runtime services

Delivered:

- payload-aware command dispatch from menus and toolbar items
- structured runtime diagnostics surfaced in the existing diagnostics catalog/UI
- optional service registry with host catalogs and service payload lookup
- host event subscriptions and emitted lifecycle/runtime events
- example plugin exercising services and host event subscription

### 6. Polish And Packaging

Outcome:

- downstream plugin author workflow is now documented and repeatable

Delivered:

- plugin discovery conventions through explicit paths, discovery directories, and default discovery roots
- platform-specific library artifact expectations documented for Linux, macOS, and Windows
- plugin author workflow guide and updated README examples
- explicit `maruzzella_api` versioning expectations in docs
- SDK ergonomics cleanup for typed JSON payload/config/service helpers

## Post-Roadmap Follow-Up

The original roadmap phases are now implemented. The next useful work should be framed as stabilization and adoption follow-up rather than another numbered roadmap phase.

Current likely targets:

- review the plugin/runtime surface for rough edges before treating it as stable for downstream products
- review the semantic appearance surface before treating its built-in ids as long-term stable product contract
- improve packaging/install conventions beyond raw platform-native libraries
- keep tightening SDK ergonomics as real third-party plugin examples appear

## Notes On Sequencing

The order above is intentional:

- the shell must feel real before deeper platform work is worth standardizing
- contribution surfaces should be derived from working shell flows rather than invented in isolation
- plugin manager and settings become much more valuable once those shared surfaces are explicit

The exact slice boundaries may still shift, but changes should continue to preserve a clean path toward this roadmap.
