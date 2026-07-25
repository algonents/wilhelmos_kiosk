//! The [`KioskApp`] lifecycle trait, the [`Kiosk`] runner, and [`KioskError`].

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::context::Context;
use crate::event::{Action, Event, Key, Mods, MouseButton};
use crate::ui::Ui;
use wilhelm_renderer::core::{Camera2D, CameraController, Color, Renderer, Window};
use wilhelm_renderer_imgui::ImGui;

/// The application lifecycle. Implement this for your kiosk application —
/// and hold your state as plain fields: every method receives `&mut self`,
/// so no `Rc<RefCell<..>>` plumbing is ever needed.
///
/// All methods are optional; a struct with an empty `impl KioskApp` block is
/// a valid (blank) application.
///
/// The shipped components ([`crate::Clock`], [`crate::StatusBar`],
/// [`crate::FpsOverlay`]) implement this same trait. Compose them by
/// embedding: store the component as a field and delegate to it from your
/// own methods (`self.clock.update(ctx, dt)`). There is no registration
/// machinery and no framework-imposed ordering — your code decides.
///
/// # FFI note
///
/// This trait is deliberately object-safe, generic-free, and uses only
/// concrete borrowed parameter types, so that a future version of the
/// framework can offer it across a C ABI (shell-binary + plugin model)
/// without redesigning the API. See `docs/DESIGN.md` §4.
pub trait KioskApp {
    /// Called once, after the window, GL context, renderer, and ImGui are
    /// live, before the first frame. Load fonts, create shapes, connect
    /// data sources here.
    fn init(&mut self, ctx: &mut Context) -> Result<(), KioskError> {
        let _ = ctx;
        Ok(())
    }

    /// Called once per frame, before drawing. `dt` is the wall-clock time
    /// in seconds since the previous `update`. Mutate shapes (via
    /// [`Context::shape_mut`]), drain data-feed channels, and advance
    /// animations here.
    fn update(&mut self, ctx: &mut Context, dt: f32) {
        let _ = (ctx, dt);
    }

    /// Called once per frame after the managed shape store has rendered and
    /// before the ImGui pass — the place for custom draw calls against
    /// [`Context::renderer`] (text runs, meshes, camera-projected world
    /// content). Draw calls made anywhere else land before the frame is
    /// cleared and vanish.
    fn draw(&mut self, ctx: &mut Context) {
        let _ = ctx;
    }

    /// Called once per frame between `ImGui::new_frame` and `ImGui::render`.
    /// Build ImGui chrome here through the guard-railed [`Ui`] wrapper.
    /// Never call `new_frame`/`render` yourself — the framework owns the
    /// frame sandwich.
    fn ui(&mut self, ui: &Ui<'_>, ctx: &mut Context) {
        let _ = (ui, ctx);
    }

    /// Called for each input event, already capture-filtered: key events are
    /// suppressed while ImGui wants the keyboard, mouse-button and scroll
    /// events while ImGui wants the mouse. See `docs/DESIGN.md` §6.
    fn on_event(&mut self, event: &Event, ctx: &mut Context) {
        let _ = (event, ctx);
    }

    /// Called once on clean exit — window close, [`Context::request_exit`],
    /// or SIGTERM/SIGINT — before the GL context is torn down. Not called
    /// after a panic (application state is unknown; the process exits
    /// nonzero and systemd's `Restart=on-failure` supervises).
    fn shutdown(&mut self, ctx: &mut Context) {
        let _ = ctx;
    }
}

/// Errors surfaced by [`Kiosk::run`].
#[derive(Debug)]
pub enum KioskError {
    /// Window, GL, renderer, or ImGui initialization failed.
    Init(String),
    /// The application reported an error from [`KioskApp::init`] (or, in a
    /// future version, from a fallible frame path).
    App(String),
    /// The application panicked during a frame. The panic message has
    /// already been logged via the panic hook; the process should exit
    /// nonzero so the systemd unit restarts the session.
    AppPanic,
}

impl std::fmt::Display for KioskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KioskError::Init(msg) => write!(f, "kiosk initialization failed: {msg}"),
            KioskError::App(msg) => write!(f, "application error: {msg}"),
            KioskError::AppPanic => write!(f, "application panicked during a frame"),
        }
    }
}

impl std::error::Error for KioskError {}

/// Builder + runner. Owns the fullscreen window, the frame loop, and the
/// init-order invariant (window callbacks → ImGui → renderer → app init).
///
/// ```no_run
/// # use wilhelmos_kiosk::{Kiosk, KioskApp, KioskError, Color};
/// # #[derive(Default)] struct MyApp;
/// # impl KioskApp for MyApp {}
/// # fn main() -> Result<(), KioskError> {
/// Kiosk::new("My Kiosk")
///     .background(Color::from_rgb(0.1, 0.1, 0.15))
///     .run(MyApp::default())
/// # }
/// ```
pub struct Kiosk {
    title: String,
    background: Color,
    camera: Option<Camera2D>,
    camera_smoothness: Option<f32>,
    camera_zoom_limits: (Option<f32>, Option<f32>),
    target_fps: Option<u32>,
}

impl Kiosk {
    /// Start configuring a kiosk session. The window is always fullscreen
    /// on the primary monitor at its current video mode.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            background: Color::from_rgb(0.0, 0.0, 0.0),
            camera: None,
            camera_smoothness: None,
            camera_zoom_limits: (None, None),
            target_fps: None,
        }
    }

    /// Background clear color (default: black).
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Enable a pan/zoom world camera. The framework wires the camera
    /// through its own unified input callbacks and blocks camera input
    /// automatically whenever ImGui captures the mouse — the
    /// `enable_camera`-clobbers-your-callbacks trap of the raw stack does
    /// not exist here.
    pub fn with_camera(mut self, camera: Camera2D) -> Self {
        self.camera = Some(camera);
        self
    }

    /// Camera smoothing factor (only meaningful with [`Kiosk::with_camera`]).
    pub fn camera_smoothness(mut self, smoothness: f32) -> Self {
        self.camera_smoothness = Some(smoothness);
        self
    }

    /// Camera zoom limits (only meaningful with [`Kiosk::with_camera`]).
    pub fn camera_zoom_limits(mut self, min: Option<f32>, max: Option<f32>) -> Self {
        self.camera_zoom_limits = (min, max);
        self
    }

    /// Optional frame pacing. `Some(fps)` sleeps the remainder of each
    /// frame budget — recommended for 24/7 operation on integrated
    /// graphics. `None` (default) renders as fast as the swap interval
    /// allows.
    pub fn target_fps(mut self, fps: Option<u32>) -> Self {
        self.target_fps = fps;
        self
    }

    /// Run the application to completion.
    ///
    /// Owns the entire session lifecycle, in order: install panic hook and
    /// signal handlers → create the fullscreen window → register framework
    /// input callbacks (before ImGui, so GLFW callback chaining works) →
    /// create ImGui → create the renderer and [`Context`] → `app.init` →
    /// frame loop (see `docs/DESIGN.md` §3 for the fixed frame order) →
    /// `app.shutdown` → ordered teardown (ImGui before window).
    ///
    /// Returns when the app requests exit, the window closes, or a
    /// SIGTERM/SIGINT arrives; returns [`KioskError::AppPanic`] if the
    /// application panicked (callers should exit nonzero so systemd's
    /// `Restart=on-failure` takes over).
    pub fn run(self, mut app: impl KioskApp) -> Result<(), KioskError> {
        let Kiosk {
            title,
            background,
            camera,
            camera_smoothness,
            camera_zoom_limits,
            target_fps,
        } = self;

        crate::log::install_panic_hook();
        crate::signal::install();

        // The Box<Window> is FFI-load-bearing (GLFW user pointer) and must
        // never be moved out; it lives on this stack frame for the whole
        // session and is dropped last (declared first).
        let mut window = Window::new_fullscreen(&title, background);
        let size = (window.width(), window.height());

        // Window callbacks are 'static, so events cross into the loop via a
        // shared queue — the one piece of Rc plumbing the framework exists
        // to hide. Registered BEFORE ImGui::new so ImGui's GLFW backend
        // chains them (see docs/DESIGN.md §3).
        let events: Rc<RefCell<VecDeque<Event>>> = Rc::new(RefCell::new(VecDeque::new()));
        {
            let q = Rc::clone(&events);
            window.on_key(move |key, _scancode, action, mods| {
                q.borrow_mut().push_back(Event::Key {
                    key: Key(key),
                    action: Action(action),
                    mods: Mods(mods),
                });
            });
            let q = Rc::clone(&events);
            window.on_mouse_button(move |button, action, mods| {
                q.borrow_mut().push_back(Event::MouseButton {
                    button: MouseButton(button),
                    action: Action(action),
                    mods: Mods(mods),
                });
            });
            let q = Rc::clone(&events);
            window.on_cursor_position(move |x, y| {
                q.borrow_mut().push_back(Event::CursorPos { x, y });
            });
            let q = Rc::clone(&events);
            window.on_scroll(move |x, y| {
                q.borrow_mut().push_back(Event::Scroll { x, y });
            });
            let q = Rc::clone(&events);
            window.on_resize(move |width, height| {
                q.borrow_mut().push_back(Event::Resize { width, height });
            });
        }

        // ImGui after callback registration; declared after `window` so its
        // Drop (ImGui/GL backend shutdown) runs before the window's.
        let imgui = ImGui::new(window.glfw_window_ptr(), true);

        let renderer = Renderer::new(window.handle());

        let camera_ctrl = camera.map(|cam| {
            let mut ctrl = CameraController::new(cam);
            if let Some(s) = camera_smoothness {
                ctrl.set_smoothness(s);
            }
            let (min, max) = camera_zoom_limits;
            if min.is_some() || max.is_some() {
                ctrl.set_zoom_limits(min, max);
            }
            ctrl
        });

        let mut ctx = Context::new(renderer, size, camera_ctrl);
        app.init(&mut ctx)?;

        // The frame loop runs under catch_unwind: a panicking app is
        // logged (panic hook), skipped past shutdown (state unknown), and
        // surfaced as AppPanic → nonzero exit → systemd Restart=on-failure.
        let loop_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut last_time = ctx.time();
            while !ctx.exit_requested()
                && !crate::signal::should_exit()
                && !window.window_should_close()
            {
                let frame_start = ctx.time();
                let dt = (frame_start - last_time) as f32;
                last_time = frame_start;

                // Dispatch queued input, capture-filtered against ImGui.
                let want_keyboard = imgui.want_capture_keyboard();
                let want_mouse = imgui.want_capture_mouse();
                loop {
                    let event = events.borrow_mut().pop_front();
                    let Some(event) = event else { break };
                    match event {
                        Event::Key { .. } if want_keyboard => continue,
                        Event::MouseButton { .. } | Event::Scroll { .. } if want_mouse => {
                            continue
                        }
                        Event::Resize { width, height } => {
                            ctx.set_size((width, height));
                        }
                        _ => {}
                    }
                    ctx.feed_camera(&event);
                    app.on_event(&event, &mut ctx);
                }

                ctx.tick_camera(dt);
                ctx.tick_fps();
                app.update(&mut ctx, dt);

                window.clear_color();
                ctx.render_shapes();
                app.draw(&mut ctx);

                imgui.new_frame();
                let ui = Ui::new(&imgui);
                app.ui(&ui, &mut ctx);
                imgui.render();

                if let Some(fps) = target_fps {
                    let budget = 1.0 / f64::from(fps.max(1));
                    let elapsed = ctx.time() - frame_start;
                    if elapsed < budget {
                        std::thread::sleep(std::time::Duration::from_secs_f64(budget - elapsed));
                    }
                }

                window.swap_buffers();
                window.poll_events();
            }
        }));

        match loop_result {
            Ok(()) => {
                app.shutdown(&mut ctx);
                crate::log::info("kiosk session exiting cleanly");
                Ok(())
            }
            Err(_) => Err(KioskError::AppPanic),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The trait must stay object-safe (FFI-promotable, DESIGN.md §4).
    #[allow(dead_code)]
    fn assert_object_safe(_app: &dyn KioskApp) {}
}
