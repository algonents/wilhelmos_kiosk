# wilhelmos_kiosk — Design

Status: v0.1.0 — frame loop, events, `Ui` guardrails, runtime services and
`FpsOverlay` implemented and smoke-tested (fullscreen run + SIGTERM clean
exit); `Clock` and `StatusBar` still stubbed (`todo!()`). This document is
the authoritative record of what the crate is,
what it refuses to be, and why each decision was taken. Style and
decision-log conventions follow `wilhelm_renderer/docs/DESIGN.md`.

## 1. Positioning

`wilhelmos_kiosk` is an **opinionated application framework** for building
fullscreen kiosk applications on WilhelmOS, layered strictly above
`wilhelm_renderer` (2D OpenGL engine) and `wilhelm_renderer_imgui` (Dear
ImGui binding). It exists because both real consumers of the raw stack —
the WilhelmOS reference app (`kiosk-app-demo`) and the production
situation display (`sky_guard_client`) — independently rewrite the same
boilerplate: window/App/ImGui initialization ordering, `Rc<RefCell<T>>`
state shared between the two callback closures (the 67-line demo needs six
bindings for three values), the ImGui frame sandwich, want-capture wiring,
an FPS counter — and neither has any error handling, panic reporting, or
SIGTERM handling, despite running under a supervising systemd unit.

The framework absorbs that boilerplate behind one lifecycle trait
([`KioskApp`]) and guards against the classic immediate-mode-GUI design
errors (§6-§7).

Deliberate scope boundaries *(decided 2026-07-25)*:

- **Pure library, in-process, no new system moving parts.** The WilhelmOS
  platform contracts (`cage-kiosk.service`, `/usr/libexec/kiosk-app`,
  `KIOSK_APP`, `virtual/kiosk-app` — wilhelmos `docs/DESIGN.md` §7) are
  untouched. A wilhelmos_kiosk app is an ordinary kiosk-app package.
- **Use it or omit it.** Integrators may keep building on raw
  `wilhelm_renderer::core::App`. Nothing in the platform or the sibling
  crates assumes this framework.
- **Link-time composition.** The integrator's binary links this crate and
  implements a trait. Runtime plugin loading was considered and rejected
  for v1 (Rust has no stable ABI; a C-ABI plugin boundary is real ongoing
  engineering with no containment payoff — see §2 and §4).
- **Not a widget toolkit, not a data-distribution layer.** Widgets are Dear
  ImGui's job; domain symbology is `wilhelm_renderer_symbols`' job; data
  feeds stay application-side (v2 will add a bridge *helper*, §13).

## 2. Certification context

WilhelmOS positions itself as COTS under ED-109A §12.4; this crate is a
**COTS library** in the same sense — evidence (tests, this document,
configuration management via pinned releases) attaches to the crate, and
the integrator's application remains the applicant's own ED-109A scope.

Two consequences shape the design:

- **Supervised robustness, not in-process recovery** *(decided
  2026-07-25)*. A library sharing the process cannot contain the
  application's faults. The honest posture: detect, report, and hand off
  to the supervisor. The framework installs a panic hook that logs at
  journald priority `<2>` with location, wraps the frame loop in
  `catch_unwind`, converts a panic into `KioskError::AppPanic` and a
  nonzero exit, and lets the platform's `Restart=on-failure` do the
  recovery. `KioskApp::shutdown` is *not* called after a panic —
  application state is unknown and pretending otherwise would be false
  assurance.
- **Dependency policy inherited from the renderer** ("dependencies must be
  minimal and auditable; implement directly rather than pull crates"):
  this crate has **zero external dependencies** — exactly
  `wilhelm_renderer` + `wilhelm_renderer_imgui`. Concrete refusals: no
  `chrono` (§8 clock uses `std::time` + hand-rolled civil-time math), no
  `log` (§9), no `libc` (§9 signals), no async runtime (§12).

## 3. The framework owns the frame loop *(decided 2026-07-25)*

The single most consequential decision. `wilhelm_renderer::core::App::run`
was examined and deliberately **not** reused: it consumes `self` with no
shutdown hook, exposes no `dt` to callbacks, offers exactly one
`on_pre_render` and one `on_render` slot (replace-on-set), has no frame
pacing, and provides no path to programmatic exit (the safe API has no
`Window::set_should_close`). None of that is fixable from outside the
loop.

Feasibility was verified against the public API: `Renderer::new(handle)`
is public (no `App` required), `CameraController` is fully public, and
shape rendering goes through the public `Renderable` trait. The framework
therefore re-implements the ~40-line loop over `Window` + `Renderer`
directly. `App` remains the supported non-framework path; if the renderer
later grows dt/shutdown hooks, this crate can re-converge.

What owning the loop buys, concretely:

- `dt` delivered to `update` (the raw loop computes it, then feeds it only
  to the camera controller).
- **Programmatic exit today**: the loop condition is
  `!exit_requested && !signal::should_exit() && !window.window_should_close()`,
  so `Context::request_exit()` and SIGTERM-clean-exit need no upstream
  change.
- Ordered shutdown: `app.shutdown(ctx)` → drop ImGui (its `Drop` must run
  before GL teardown) → drop window.
- Optional frame pacing (`Kiosk::target_fps`) for 24/7 operation on
  integrated graphics.
- A place to auto-wire capture filtering and the camera block flag (§6).

Fixed frame order (one frame):

```
dt ← now − last
exit checks (request_exit | SIGTERM/SIGINT | window close) → shutdown path
poll-side events already dispatched → app.on_event (capture-filtered, §6)
camera_block ← imgui.want_capture_mouse()
camera_controller.update(dt)
app.update(ctx, dt)
clear
render shape store in z-order (§5)
app.draw(ctx)              ← custom renderer calls (world content, text runs)
imgui.new_frame
  app.ui(ui, ctx)          ← only place ImGui widgets may be built
imgui.render
[optional] sleep(frame_budget − elapsed)      target_fps pacing
swap_buffers
poll_events
```

Init-order invariant, owned by `Kiosk::run` so integrators cannot get it
wrong: panic hook + signal handlers → fullscreen window
(`Window::new_fullscreen`; the `Box<Window>` never moves — its address is
GLFW's user pointer) → **framework window callbacks registered before**
`ImGui::new(ptr, true)` (GLFW callback chaining requires this order) →
`Renderer::new` → `Context` → `app.init`.

Threading: the entire stack is single-threaded by design (thread-local
shaders in the renderer, `ImGui` is `!Send + !Sync`). The framework runs
everything on the main thread and documents it; background work
communicates via channels (§13).

## 4. The `KioskApp` trait *(decided 2026-07-25)*

One uniform, object-safe trait with defaulted methods:

```rust
trait KioskApp {
    fn init(&mut self, ctx: &mut Context) -> Result<(), KioskError>;
    fn update(&mut self, ctx: &mut Context, dt: f32);
    fn draw(&mut self, ctx: &mut Context);   // after shape pass, before UI
    fn ui(&mut self, ui: &Ui<'_>, ctx: &mut Context);
    fn on_event(&mut self, event: &Event, ctx: &mut Context);
    fn shutdown(&mut self, ctx: &mut Context);
}
```

- **`&mut self` everywhere** is the state-management fix: application
  state is plain struct fields. The `Rc<RefCell<T>>`-per-scalar pattern
  the raw stack forces (state shared between two `'static` closures)
  disappears entirely. This codifies what sky_guard already converged on
  by hand (its `Scene::draw_frame(&mut self, …)`).
- **One trait for apps and components alike.** The shipped components
  (§8) implement the same trait; composition is embedding + delegation
  (`self.clock.update(ctx, dt)`) — no registration lists, no
  framework-imposed ordering, nothing to configure. The application
  decides what runs and when, in ordinary Rust.
- **FFI-promotable by construction**: no generics on the trait, only
  concrete borrowed parameter types, handle-based context (§5). If the
  "certified shell binary + customer plugin" model ever becomes
  commercially decisive, this trait can be promoted across a C ABI
  (vtable + plugin-side wrapper crate) without an API redesign. That work
  is explicitly deferred — the trait shape is the only provision made.

## 5. `Context` and the shape store

`Context` is the framework services handle passed to every trait method:
renderer access, shape store, camera, frame stats (`fps()` — the EWMA
counter every consumer hand-rolls today), `size()`, `time()`, and
`request_exit()`.

**Stable `ShapeId` handles** *(decided 2026-07-25)*: `App` stores shapes
in a `Vec` it re-sorts by z-order **in place** every frame, so the slice
indices its `on_pre_render` callback receives are not stable across frames
— the reference demo survives only because it has exactly one shape. The
framework's store maps `ShapeId → ShapeRenderable` (insertion-stable;
z-order is a separately-built render index, the store never reorders) and
applications address shapes only through handles. Handle-based access is
also part of the FFI-promotability story (§4).

`WindowHandle` is not publicly nameable in `wilhelm_renderer` 0.13 (the
`window` module is private and the type is not re-exported), so `Context`
tracks the framebuffer size itself, updated from resize events. Upstream
issue filed (§11).

## 6. Events and capture filtering

The raw stack exposes five separate `'static` closures taking raw `i32`
GLFW codes, with no letter/digit constants exported (sky_guard compares
against a literal `79` for the letter O). The framework folds these into
one typed `Event` enum dispatched to `on_event`.

Design choices:

- **Newtypes over raw `i32`** (`Key`, `MouseButton`, `Action`, `Mods`)
  with named associated constants, not Rust enums: unknown codes pass
  through losslessly, new GLFW keys are never a breaking change, and the
  values are checkable against the renderer's re-exported constants (unit
  tests do). Letters/digits use GLFW's stable ASCII values, defined here.
- **Capture filtering is automatic** *(decided 2026-07-25)*: key events
  are suppressed while `want_capture_keyboard()`, mouse-button and scroll
  events while `want_capture_mouse()`. Today every app must remember the
  `if !imgui.want_capture_*` guard; forgetting it is the classic
  click-through bug. Cursor moves are still delivered (hover tracking is
  legitimate).
- **The camera block flag is auto-wired**: the framework sets it from
  `want_capture_mouse()` every frame — the manual wiring step the raw
  stack documents but its own reference demo forgets. Because the
  framework registers the camera controller through its own unified
  callbacks, the `enable_camera`-clobbers-your-callbacks trap of the raw
  stack does not exist here.
- Candidate for upstreaming later; for now the event layer lives entirely
  in this crate.

## 7. ImGui guardrails

Immediate-mode errors this crate makes hard or impossible:

| Pitfall | Guardrail |
|---|---|
| Unbalanced `begin`/`end` (incl. `begin`→`false` still needing `end`) | `Ui::window(title, flags, closure)` owns the pairing; `end` always runs |
| `imgui.ini` written to the appliance rootfs | `NO_SAVED_SETTINGS` OR-ed into every `Ui::window` unconditionally |
| Widget ID collisions in loops | `Ui::with_id(id, closure)` scoped push/pop |
| Unpaired `new_frame`/`render` | Owned by the frame loop; applications never call them (`Ui` does not expose them) |
| UI built outside the frame sandwich | Widgets only reachable through the `Ui` handle passed to `KioskApp::ui`, which is invoked inside the sandwich |
| State mutated by UI mid-frame racing the draw pass | `ui` runs after `update` and after shape rendering in the fixed frame order (§3); sliders take `&mut self` fields, applied next `update` |

`Ui::raw()` deliberately exposes the underlying `&ImGui` — the full widget
set stays reachable, the wrapper adds scoping, not a walled garden.
`Ui::window_at` covers the fixed-position chrome case (status bars,
overlays) via `set_next_window_pos/size`.

## 8. Components v1

Ordinary `KioskApp` implementations, shipped in `src/components/`:

- **`Clock`** — UTC `HH:MM:SS`. Time from `std::time::SystemTime` with
  hand-rolled civil-time conversion (*no `chrono`*, §2; UTC only — ground
  equipment runs UTC and time zones are complexity this crate refuses).
  Rendered with the renderer's TrueType path (`FontAtlas` + `Text`
  shapes), **not** ImGui text: the bundled ImGui cannot load custom fonts
  (§11), and aviation chrome needs real display fonts (B612 Mono). Font
  path is a constructor parameter — the crate bundles no fonts.
- **`StatusBar`** — full-width bar (renderer rect + left/center/right text
  slots, right-alignment via `FontAtlas::measure_text`), anchored top or
  bottom. Composes with `Clock`.
- **`FpsOverlay`** — small ImGui overlay reading `Context::fps()`;
  commissioning aid, not production chrome.

**Explicitly no terminal component** *(decided 2026-07-25)*: maintenance
access on WilhelmOS stays on tty2, behind a login, outside the kiosk
session. An in-kiosk terminal would change the product's security posture
(arbitrary command execution from the operator seat, in-process with the
display application) for no operational need a status display has.

## 9. Runtime services

- **Logging** — three free functions (`log::info/warn/error`) writing
  sd-daemon `<N>`-prefixed lines to stderr; the kiosk unit already routes
  stderr to journald with per-line priorities. No `log`/`tracing` crate,
  no initialization, nothing to misconfigure.
- **Panic hook** — priority `<2>` with file:line:column and payload,
  chained onto the default hook; installed by `Kiosk::run` (§2).
- **Signals** *(decided 2026-07-25)* — SIGTERM/SIGINT handler stores one
  `AtomicBool` (the only async-signal-safe action worth taking); the loop
  polls it each frame and exits through the clean shutdown path, so
  `systemctl stop` gives the app its `shutdown` callback. Implemented
  against `signal(2)` declared by hand from the libc `std` already links
  — the `libc` crate was weighed and **rejected** to hold the zero-dep
  line; `sigaction` (SA_RESTART control) deferred until a concrete need
  appears.

## 10. Packaging & versioning

- **First crate to depend on both siblings.** `wilhelm_renderer` pins
  `wilhelm_renderer_sys = "=0.11.0"` and carries `links =
  "wilhelm_renderer"`; `wilhelm_renderer_imgui` depends on sys `0.11`.
  Both must resolve to a single sys version or cargo fails on the links
  collision. **Rule: any bump of one dependency moves the other in
  lockstep** (recorded in `Cargo.toml`). Current floor: renderer
  `0.13.0`, imgui `0.10.0`, edition 2021 (family-wide).
- **No cargo features.** The family's build switches are environment
  variables consumed by the sys build scripts (`GLRENDERER_BUILD_X11`,
  `GLRENDERER_LINK_GL`); this crate adds none and passes those through
  untouched — the WilhelmOS recipe pattern (`kiosk-app-demo_git.bb`)
  works unchanged for a wilhelmos_kiosk app.
- **Release rule (family-wide):** a tagged release must have zero git
  dependencies in `Cargo.lock` — integrator apps are built by Yocto's
  `cargo-update-recipe-crates` flow, which fetches every locked crate
  individually from crates.io.
- Versioning: pre-1.0 semver; the trait surface (§4) is the compatibility
  contract to protect.

## 11. Upstream gaps

Worked around in v1 (no upstream change required):

| Gap (upstream) | v1 workaround here |
|---|---|
| No `Window::set_should_close` | Owned loop condition + `request_exit()` |
| No `dt` to callbacks, no shutdown hook, `run` consumes self | Loop replaced (§3) |
| `enable_camera` clobbers window callbacks | Framework wires `CameraController` through its own callbacks |
| Callbacks-must-precede-`ImGui::new` ordering trap | Order owned inside `Kiosk::run` |
| No custom ImGui fonts (TTF) | Chrome text via renderer `FontAtlas`/`Text` (§8) |
| `imgui.ini` persistence default | `NO_SAVED_SETTINGS` forced (§7) |
| Shape-slice index instability after in-place z-sort | Stable `ShapeId` store (§5) |
| No letter/digit key constants | Defined here from the GLFW ABI (§6) |
| `WindowHandle` not publicly nameable | `Context` tracks size itself (§5) |
| No frame pacing | `target_fps` sleep in the owned loop |

To file upstream (nice-to-haves; none block this crate):

- `wilhelm_renderer`: export `WindowHandle`; `Window::set_should_close`;
  pass `dt` to `App` callbacks + an `on_shutdown` hook; full GLFW key
  constant set.
- `wilhelm_renderer`: monitor physical size (for the §12 DPI
  auto-detect fallback).
- `wilhelm_renderer_imgui`: custom TTF font loading
  (`AddFontFromFileTTF`); safe wrappers for `push_style_var_*`; an
  ini-disable binding. ~~An explicit UI-scale entry point,
  `imgui_set_ui_scale(scale)`~~ — *shipped in `wilhelm_renderer_imgui`
  0.11.0* (needed by §12, since `apply_dpi_scale` only reads GLFW's
  content scale, always 1.0 under cage).

## 12. UI scaling (DPI) *(agreed & implemented 2026-07-26; meta-wilhelmos `Environment=` pending)*

**Problem.** cage maps the panel at its native mode with Wayland output
scale 1.0, so GLFW reports a content scale of 1.0 and the application
draws in raw physical pixels — on a 4K panel a 16 px glyph is ~2 mm
tall. Nothing in the stack scales anything today:
`wilhelm_renderer_imgui`'s existing `apply_dpi_scale` is inert here (it
reads GLFW's content scale, which is always 1.0 under cage; it was
written for Windows and only rebuilds ImGui's default 13 px bitmap font
anyway).

**Option A — compositor-side scaling (rejected).** cage has no scale
flag, but it implements `wlr-output-management`, so the image could ship
`wlr-randr` and set output scale at session start. Rejected because:
integer scales only (2× on 4K is too much; 1.5× would need the
fractional-scale protocol, which GLFW does not speak), it adds a package
to the certified image, and it splits sizing logic between compositor
and application. The variant of setting a lower video mode instead lets
the panel upscale — blurry, unacceptable for a situation display.

**Option B — framework-owned `ui_scale` (chosen).** The output stays at
native resolution, scale 1.0; this crate is the single owner of a
`ui_scale` factor, applied at the points where pixels are generated:

- **Source of truth:** a `WILHELMOS_UI_SCALE` environment variable
  (e.g. `1.5`), set per deployment via `Environment=` in
  `cage-kiosk.service` (or a drop-in) — QEMU and a 4K bare-metal box
  differ only in config. Absent/invalid ⇒ 1.0. Auto-detection from
  monitor physical size (DPI) is deferred: it needs an upstream
  `wilhelm_renderer` API (§11) and would only be a fallback when the
  variable is unset.
- **Exposure:** `Context::ui_scale()`; read by components and available
  to applications for DPI-aware marker sizes / line widths.
- **ImGui chrome:** needs a new explicit-scale entry point in
  `wilhelm_renderer_imgui` (`imgui_set_ui_scale(scale)`:
  `style.ScaleAllSizes(scale)` + rebuild the font at `13 × scale`) —
  the existing content-scale-reading one cannot be told a factor. The
  framework calls it once at init.
- **Renderer text:** freetype rasterizes at a pixel size, so the
  framework and components (§8) multiply their base font sizes by
  `ui_scale` at `FontAtlas` creation — text stays pixel-crisp at any
  fractional scale because it is rasterized at final size, never
  upscaled.
- **World content is deliberately not scaled:** camera zoom is the
  semantic scale of the situation picture; DPI scaling affects chrome
  and text only.

Work split: `wilhelm_renderer_imgui` (explicit-scale API + version
bump), this crate (`ui_scale` in `Context`, ImGui init call, component
font sizing), `meta-wilhelmos` (set the variable in the kiosk session
unit).

## 13. Roadmap (deferred beyond v1)

- **Async data-feed bridge** — the headline v2 item. sky_guard's
  production pattern: a `std::thread` hosting a tokio runtime, pushing
  into a `std::sync::mpsc` channel with a reconnect/backoff loop, drained
  non-blocking from the render loop. The trait already fits — `update` is
  the drain point; the likely shape is a `FeedHandle<T>` spawn helper plus
  a connection-state surface for chrome. The async runtime stays
  application-side; this crate stays runtime-agnostic.
- Splash / "connecting…" startup state (blocking data loads currently
  show a black screen).
- Config loading conventions.
- Cursor hiding / screen-blank inhibit for touchless installations.
- Frame-time watchdog statistics (p99 frame time) surfaced to §5 stats —
  groundwork for the platform's §2.4.3 health-monitoring story.
- C-ABI trait promotion (§4) — only if a customer pays for the
  bit-identical-shell certification argument.
