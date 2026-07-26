# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-07-26

### Added

- Migration enablers driven by the sky_guard_client port (the framework's
  first production consumer):
  - `Kiosk::imgui_config_flags(i32)` — ImGui IO config flags (e.g.
    `config_flags::NAV_ENABLE_KEYBOARD`) applied once at startup;
    `config_flags` is now re-exported alongside `window_flags`.
  - `Context::set_camera(Camera2D)` — install or replace the world camera
    from `KioskApp::init`, for cameras that depend on loaded data and the
    real screen size. Rebuilds the camera controller so its animation
    targets reset to the new state (repositioning via `camera_mut()`
    rubber-bands under smoothing — now documented on `camera_mut`).
    Builder-configured smoothness/zoom limits are re-applied.
  - `Context::set_camera_zoom_limits(min, max)` — runtime zoom limits,
    e.g. derived from data loaded in `init`.

### Removed

- `examples/hello_kiosk.rs`: the crate ships no example anymore —
  [kiosk-app-demo](https://github.com/algonents/kiosk-app-demo) is the
  single canonical reference application (DESIGN.md §1, one-app rule).
  The example's Escape→exit binding is gone with it: no key may reach
  the clean-exit path in a kiosk app (DESIGN.md §3); desktop test runs
  exit via Ctrl+C.

## [0.2.0] - 2026-07-26

### Added

- UI scaling (DPI) per `docs/DESIGN.md` §12: the `WILHELMOS_UI_SCALE`
  environment variable (accepted range 0.5–4.0; absent/invalid ⇒ 1.0 with
  a journald warning) scales the ImGui chrome at init (via the new
  `wilhelm_renderer_imgui 0.11.0` `set_ui_scale` API) and is exposed as
  `Context::ui_scale()` for components and applications. World content
  (camera) is deliberately not scaled. `FpsOverlay` scales its position;
  `Clock`/`StatusBar` will multiply their base sizes when implemented.
  Follow-up (separate repo): set the variable via `Environment=` in
  meta-wilhelmos' `cage-kiosk.service`.

### Changed

- `wilhelm_renderer_imgui` dependency: 0.10.0 → 0.11.0.

## [0.1.0] - 2026-07-26

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
- Working core (frame loop and everything it drives):
  - `Kiosk::run` implemented: owned frame loop with `dt`, optional
    `target_fps` pacing, capture-filtered event dispatch, automatic camera
    wiring, `catch_unwind` panic containment, SIGTERM/SIGINT clean exit,
    ordered teardown
  - `KioskApp::draw` lifecycle hook (custom renderer calls after the shape
    pass, before the UI pass)
  - `Context`: z-ordered shape render pass, `camera_mut`, camera
    feed/tick plumbing
  - `Ui::window` / `window_at` / `with_id` implemented (begin/end pairing,
    `NO_SAVED_SETTINGS` default, ID scoping)
  - `FpsOverlay` implemented; `hello_kiosk` grows an FPS overlay and
    Escape-to-exit
  - Smoke-tested on the dev host: fullscreen launch, hardware GL, clean
    SIGTERM shutdown (`<6>kiosk session exiting cleanly`)
