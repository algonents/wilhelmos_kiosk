# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial design (`docs/DESIGN.md`) and compiling API skeleton:
  - `KioskApp` lifecycle trait and `Kiosk` builder/runner (loop stubbed)
  - `Context` with stable `ShapeId` shape store and EWMA FPS counter
  - Typed input events (`Event`, `Key`, `MouseButton`, `Action`, `Mods`)
    over the raw GLFW codes, including letter/digit constants
  - `Ui` scoped ImGui guardrails (stubbed)
  - Journald-priority logging and panic hook; SIGTERM/SIGINT clean-exit
    flag (zero-dependency implementations)
  - Components (stubbed): `Clock` (UTC), `StatusBar`, `FpsOverlay`
  - `examples/hello_kiosk.rs` — the WilhelmOS reference demo rewritten
    against the framework
