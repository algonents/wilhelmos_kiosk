//! The [`KioskApp`] lifecycle trait, the [`Kiosk`] runner, and [`KioskError`].

use crate::context::Context;
use crate::event::Event;
use crate::ui::Ui;
use wilhelm_renderer::core::{Camera2D, Color};

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
    #[allow(dead_code)] // consumed by `run` (window title), still a stub
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
    pub fn run(self, app: impl KioskApp) -> Result<(), KioskError> {
        let _ = app;
        todo!("owned frame loop — see docs/DESIGN.md §3")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The trait must stay object-safe (FFI-promotable, DESIGN.md §4).
    #[allow(dead_code)]
    fn assert_object_safe(_app: &dyn KioskApp) {}
}
