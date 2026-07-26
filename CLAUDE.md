# wilhelmos_kiosk

Opinionated application framework for fullscreen kiosk apps, layered on
`wilhelm_renderer` + `wilhelm_renderer_imgui`. Primary deployment target:
WilhelmOS kiosk mode (`systemd → cage → app → OpenGL → DRM/KMS`).

**Status: working core.** `Kiosk::run` (owned frame loop), events, the
`Ui` scoped helpers, and `FpsOverlay` are implemented and smoke-tested;
`Clock` and `StatusBar` bodies are still `todo!()` stubs.
`docs/DESIGN.md` is the authoritative design record — read it before
changing the API; it explains every decision, including the ones that look
odd (owned loop, zero deps, no terminal component).

## Sibling repos

| Repo | Relationship |
|---|---|
| `../wilhelm_renderer` | 2D OpenGL engine underneath (dep, 0.13.x) |
| `../wilhelm_renderer_imgui` | Dear ImGui binding underneath (dep, 0.10.x) |
| `../kiosk-app-demo` | The canonical example: illustrates this crate in practice + the packaging contract. This repo ships no example of its own (single-source rule, no drift) |
| `../sky_guard` | Production consumer-to-be (its Scene/FpsCounter/feed patterns shaped this design) |
| `../wilhelmos` | The OS; its `docs/DESIGN.md` §7 defines the kiosk-app packaging contract |

## Build

```
cargo check --all-targets   # must stay warning-free
cargo test
cargo clippy --all-targets
```

No cargo features. The family's env-var build switches
(`GLRENDERER_BUILD_X11`, `GLRENDERER_LINK_GL`) belong to the sys build
scripts and pass through this crate untouched.

## Design invariants (do not break casually)

- **Zero external dependencies** — exactly the two sibling crates
  (ED-109A-driven policy, DESIGN.md §2). No chrono, no log, no libc, no
  async runtime.
- **Version lockstep**: renderer pins sys with `=`; any bump of one
  sibling dep must bump the other so a single `wilhelm_renderer_sys`
  resolves (`links` collision otherwise).
- **`KioskApp` stays object-safe and generic-free** (FFI-promotable,
  DESIGN.md §4); `Context` stays handle-based (`ShapeId`).
- The framework owns `new_frame`/`render` and the init order
  (callbacks-before-`ImGui::new`); `Ui` must never expose the frame
  sandwich.
- Tagged releases: no git deps in `Cargo.lock` (Yocto
  `cargo-update-recipe-crates` flow).

## Key files

- `src/app.rs` — `KioskApp` trait, `Kiosk` builder/runner, `KioskError`
- `src/context.rs` — `Context`, `ShapeId` store, `FpsCounter`
- `src/event.rs` — typed events over raw GLFW i32 codes (complete)
- `src/ui.rs` — scoped ImGui guardrails
- `src/log.rs`, `src/signal.rs` — journald logging + SIGTERM flag (complete)
- `src/components/` — Clock, StatusBar, FpsOverlay
- `docs/DESIGN.md` — the design record (13 sections)
- No `examples/` — `../kiosk-app-demo` is the reference application
