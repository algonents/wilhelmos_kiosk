//! Guard-railed ImGui access.
//!
//! Dear ImGui's `begin`/`end` calls must be paired manually — including the
//! non-obvious case where `begin` returns `false` (window collapsed) and
//! `end` must still be called. [`Ui`] provides closure-scoped wrappers that
//! make unbalanced pairs unrepresentable, and applies kiosk-appropriate
//! defaults (`NO_SAVED_SETTINGS`, so no `imgui.ini` is written on the
//! appliance's read-mostly rootfs).

use wilhelm_renderer_imgui::{cond, window_flags, ImGui};

/// Scoped ImGui wrapper handed to [`crate::KioskApp::ui`] each frame.
///
/// All widget calls happen on the borrowed [`ImGui`] inside the closures;
/// the wrapper only owns scoping (window begin/end, ID push/pop) and
/// defaults. Use [`Ui::raw`] to reach the full widget set directly.
pub struct Ui<'a> {
    imgui: &'a ImGui,
}

impl<'a> Ui<'a> {
    pub(crate) fn new(imgui: &'a ImGui) -> Self {
        Self { imgui }
    }

    /// Begin a window, run `body` if it is visible, and always end it —
    /// the begin/end pairing (including the collapsed case) is owned here.
    /// `window_flags::NO_SAVED_SETTINGS` is OR-ed into `flags`
    /// unconditionally.
    pub fn window(&self, title: &str, flags: i32, body: impl FnOnce(&ImGui)) {
        let visible = self
            .imgui
            .begin(title, None, flags | window_flags::NO_SAVED_SETTINGS);
        if visible {
            body(self.imgui);
        }
        // Dear ImGui requires End() even when Begin() returned false.
        self.imgui.end();
    }

    /// [`Ui::window`] with an explicit position (and optional size) applied
    /// via `set_next_window_pos`/`set_next_window_size` — the common shape
    /// for fixed kiosk chrome (status bars, overlays).
    pub fn window_at(
        &self,
        title: &str,
        pos: (f32, f32),
        size: Option<(f32, f32)>,
        flags: i32,
        body: impl FnOnce(&ImGui),
    ) {
        self.imgui.set_next_window_pos(pos.0, pos.1, cond::ALWAYS);
        if let Some((width, height)) = size {
            self.imgui.set_next_window_size(width, height, cond::ALWAYS);
        }
        self.window(title, flags, body);
    }

    /// Run `body` inside a pushed ImGui ID scope — the escape hatch for
    /// building repeated widgets in a loop without label collisions.
    pub fn with_id(&self, id: &str, body: impl FnOnce(&ImGui)) {
        self.imgui.push_id(id);
        body(self.imgui);
        self.imgui.pop_id();
    }

    /// The underlying [`ImGui`] handle, for widget calls outside a scoped
    /// helper. The frame sandwich (`new_frame`/`render`) is still owned by
    /// the framework — never call those.
    pub fn raw(&self) -> &ImGui {
        self.imgui
    }
}
