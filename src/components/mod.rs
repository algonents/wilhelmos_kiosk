//! Predefined kiosk components.
//!
//! Each component is an ordinary [`crate::KioskApp`] implementation.
//! Compose by embedding: hold the component as a field of your application
//! and delegate the lifecycle calls you want it to receive
//! (`self.clock.update(ctx, dt)`, `self.clock.ui(ui, ctx)`, …). There is no
//! registration list and no framework-imposed ordering.
//!
//! Deliberately absent: a terminal. Maintenance access on WilhelmOS stays
//! on tty2, outside the kiosk session (`docs/DESIGN.md` §8).

mod clock;
mod fps_overlay;
mod status_bar;

pub use clock::Clock;
pub use fps_overlay::FpsOverlay;
pub use status_bar::StatusBar;
