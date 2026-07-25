# wilhelmos_kiosk

Opinionated application framework for fullscreen kiosk applications, built
on [wilhelm_renderer](https://github.com/algonents/wilhelm_renderer) and
Dear ImGui (via
[wilhelm_renderer_imgui](https://github.com/algonents/wilhelm_renderer_imgui)).
Designed for [WilhelmOS](https://github.com/algonents/wilhelmos) kiosk
deployments (`systemd → cage → your app → OpenGL → DRM/KMS → display`), but
runs on any desktop the underlying stack supports.

> **Status: design + API skeleton.** The public API is in place and
> compiles; frame-loop and component implementations are stubbed. See
> [docs/DESIGN.md](docs/DESIGN.md) for the full architecture and rationale.

## Why

Applications built on the raw stack all rewrite the same boilerplate:
init ordering, `Rc<RefCell<..>>` state shared between callbacks, the ImGui
frame sandwich, capture-filter wiring, FPS counting — and usually ship with
no panic reporting or SIGTERM handling despite running under systemd
supervision. `wilhelmos_kiosk` absorbs all of that behind one lifecycle
trait:

- **`KioskApp` trait** — `init` / `update(dt)` / `ui` / `on_event` /
  `shutdown`, all `&mut self`: state is plain struct fields.
- **Owned frame loop** — fixed frame order, `dt`, optional frame pacing,
  programmatic + signal-driven clean exit, ordered teardown.
- **Typed input events** — `Key::O` instead of a magic `79`; ImGui
  capture-filtering applied automatically.
- **ImGui guardrails** — closure-scoped `begin`/`end` pairing, no
  `imgui.ini` writes, ID scoping.
- **Supervised robustness** — journald-priority logging, panic hook,
  `catch_unwind` → nonzero exit → systemd `Restart=on-failure`.
- **Components** — `Clock` (UTC), `StatusBar`, `FpsOverlay`; ordinary
  `KioskApp` impls you embed and delegate to. No terminal, by design.
- **Zero external dependencies** — exactly the two sibling crates.

## Quick start

```toml
[dependencies]
wilhelmos_kiosk = "0.1"
```

```rust,no_run
use wilhelmos_kiosk::{Color, Context, Kiosk, KioskApp, KioskError, Ui};

#[derive(Default)]
struct MyApp {
    brightness: f32,
}

impl KioskApp for MyApp {
    fn update(&mut self, _ctx: &mut Context, _dt: f32) {}

    fn ui(&mut self, ui: &Ui<'_>, _ctx: &mut Context) {
        ui.window("Controls", 0, |im| {
            im.slider_float("Brightness", &mut self.brightness, 0.0, 1.0);
        });
    }
}

fn main() -> Result<(), KioskError> {
    Kiosk::new("My Kiosk")
        .background(Color::from_rgb(0.1, 0.1, 0.15))
        .run(MyApp::default())
}
```

See [`examples/hello_kiosk.rs`](examples/hello_kiosk.rs) — the WilhelmOS
reference demo rewritten against the framework, with zero `Rc<RefCell<..>>`.

## Build requirements

Same as the underlying stack (CMake + native GLFW/FreeType/ImGui builds;
on Linux: `libgl1-mesa-dev libwayland-dev libxkbcommon-dev xorg-dev`). The
family's environment-variable build switches pass through untouched —
Wayland-only embedded builds use `GLRENDERER_BUILD_X11=OFF` and
`GLRENDERER_LINK_GL=OFF` exactly as before.

## Issues

https://github.com/algonents/wilhelmos_kiosk/issues

## License

MIT © 2025 Algonents
