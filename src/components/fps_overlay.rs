//! Frame-rate overlay component.

use crate::app::KioskApp;
use crate::context::Context;
use crate::ui::Ui;
use wilhelm_renderer_imgui::window_flags;

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
        let fps = ctx.fps();
        let flags = window_flags::NO_TITLE_BAR
            | window_flags::NO_RESIZE
            | window_flags::NO_MOVE
            | window_flags::ALWAYS_AUTO_RESIZE
            | window_flags::NO_MOUSE_INPUTS;
        ui.window_at("fps_overlay", (12.0, 12.0), None, flags, |im| {
            im.text(&format!("{fps:5.1} FPS"));
        });
    }
}
