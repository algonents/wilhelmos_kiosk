//! Status-bar component.

use crate::app::{KioskApp, KioskError};
use crate::context::Context;

/// A full-width status bar (renderer-drawn rectangle plus three text
/// slots: left, center, right) anchored to the top or bottom edge of the
/// display. Text renders through the renderer's TrueType path, same as
/// [`crate::Clock`] — with which it composes naturally (put a `Clock` in
/// one slot, or embed both and let the bar own the clock text).
pub struct StatusBar {
    font_path: String,
    height_px: f32,
    left: String,
    center: String,
    right: String,
}

impl StatusBar {
    /// A status bar `height_px` tall, rendering text with the TrueType
    /// font at `font_path`. Fonts load in `init` (GL context required).
    ///
    /// `height_px` is a *base (unscaled)* height: `init` multiplies it —
    /// and the derived font size — by [`Context::ui_scale`]
    /// (`docs/DESIGN.md` §12).
    pub fn new(font_path: &str, height_px: f32) -> Self {
        Self {
            font_path: font_path.to_string(),
            height_px,
            left: String::new(),
            center: String::new(),
            right: String::new(),
        }
    }

    /// Set the left-aligned text slot.
    pub fn set_left(&mut self, text: &str) {
        self.left = text.to_string();
    }

    /// Set the center text slot.
    pub fn set_center(&mut self, text: &str) {
        self.center = text.to_string();
    }

    /// Set the right-aligned text slot (laid out via
    /// `FontAtlas::measure_text`).
    pub fn set_right(&mut self, text: &str) {
        self.right = text.to_string();
    }
}

impl KioskApp for StatusBar {
    fn init(&mut self, ctx: &mut Context) -> Result<(), KioskError> {
        let _ = ctx;
        todo!(
            "background rect ({}px * ctx.ui_scale()) + FontAtlas from {:?} \
             at a ui_scale-multiplied size — DESIGN.md §8, §12",
            self.height_px,
            self.font_path
        )
    }

    fn update(&mut self, ctx: &mut Context, dt: f32) {
        let _ = (ctx, dt);
        todo!("re-layout text slots when contents change — DESIGN.md §8")
    }
}
