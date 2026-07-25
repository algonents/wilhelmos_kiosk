//! # wilhelm_kiosk
//!
//! Opinionated application framework for fullscreen kiosk applications
//! built on [`wilhelm_renderer`] and Dear ImGui (via `wilhelm_renderer_imgui`).
//!
//! The framework owns the window, the frame loop, and the ImGui frame
//! sandwich; your application implements the [`KioskApp`] lifecycle trait
//! and holds its state as plain `&mut self` fields — no `Rc<RefCell<..>>`
//! sharing between closures.
//!
//! ```no_run
//! use wilhelm_kiosk::{Color, Context, Kiosk, KioskApp, KioskError, Ui};
//!
//! #[derive(Default)]
//! struct MyApp {
//!     brightness: f32,
//! }
//!
//! impl KioskApp for MyApp {
//!     fn update(&mut self, _ctx: &mut Context, _dt: f32) {}
//!
//!     fn ui(&mut self, ui: &Ui<'_>, _ctx: &mut Context) {
//!         ui.window("Controls", 0, |im| {
//!             im.slider_float("Brightness", &mut self.brightness, 0.0, 1.0);
//!         });
//!     }
//! }
//!
//! fn main() -> Result<(), KioskError> {
//!     Kiosk::new("My Kiosk")
//!         .background(Color::from_rgb(0.1, 0.1, 0.15))
//!         .run(MyApp::default())
//! }
//! ```
//!
//! See `docs/DESIGN.md` for the full architecture and the rationale behind
//! every decision.

pub mod app;
pub mod components;
pub mod context;
pub mod event;
pub mod log;
pub mod ui;

mod signal;

pub use app::{Kiosk, KioskApp, KioskError};
pub use components::{Clock, FpsOverlay, StatusBar};
pub use context::{Context, ShapeId};
pub use event::{Action, Event, Key, Mods, MouseButton};
pub use ui::Ui;

// Curated re-exports of the underlying stack, so simple applications need
// only depend on the types this crate's API surfaces. Anything else (shape
// geometry types, cameras, markers, …) is available from the sibling crates
// directly — both are public dependencies of this one.
pub use wilhelm_renderer::core::{Camera2D, Color, Renderer};
pub use wilhelm_renderer::graphics2d::shapes::{ShapeKind, ShapeRenderable, ShapeStyle};
pub use wilhelm_renderer_imgui::{window_flags, ImGui};
