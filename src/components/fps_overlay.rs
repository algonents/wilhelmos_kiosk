//! Frame-rate overlay component.

use crate::app::KioskApp;
use crate::context::Context;
use crate::ui::Ui;

/// A small ImGui overlay showing the framework's built-in EWMA FPS
/// ([`Context::fps`]) — a development/commissioning aid, typically not
/// composed into production chrome.
#[derive(Default)]
pub struct FpsOverlay {
    _private: (),
}

impl FpsOverlay {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KioskApp for FpsOverlay {
    fn ui(&mut self, ui: &Ui<'_>, ctx: &mut Context) {
        let _ = (ui, ctx);
        todo!("frameless corner window showing ctx.fps() — DESIGN.md §8")
    }
}
