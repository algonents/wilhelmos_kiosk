//! UTC wall-clock component.

use crate::app::{KioskApp, KioskError};
use crate::context::Context;

/// A UTC clock (`HH:MM:SS`), rendered with the renderer's own TrueType text
/// path (`FontAtlas` + `Text` shapes) so that real display fonts — e.g.
/// B612 Mono on an ATM position — are used. (The bundled ImGui cannot load
/// custom fonts; see `docs/DESIGN.md` §8 and §11.)
///
/// Time source is `std::time::SystemTime` with hand-rolled civil-time
/// conversion — no `chrono`, per the family dependency policy. UTC only:
/// ground-equipment displays run UTC, and time zones are exactly the kind
/// of complexity this crate refuses to carry.
///
/// # Composition
///
/// ```no_run
/// # use wilhelm_kiosk::{Clock, Context, KioskApp, KioskError};
/// struct MyApp { clock: Clock }
///
/// impl KioskApp for MyApp {
///     fn init(&mut self, ctx: &mut Context) -> Result<(), KioskError> {
///         self.clock.init(ctx)
///     }
///     fn update(&mut self, ctx: &mut Context, dt: f32) {
///         self.clock.update(ctx, dt);
///     }
/// }
/// ```
pub struct Clock {
    font_path: String,
    size_px: u32,
    position: (f32, f32),
}

impl Clock {
    /// A clock rendering with the TrueType font at `font_path` at
    /// `size_px` pixels. The font file is loaded in `init` (a GL context
    /// is required to build the glyph atlas), so a bad path fails there,
    /// not here.
    pub fn new(font_path: &str, size_px: u32) -> Self {
        Self {
            font_path: font_path.to_string(),
            size_px,
            position: (0.0, 0.0),
        }
    }

    /// Screen position of the clock's top-left corner, in pixels.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }
}

impl KioskApp for Clock {
    fn init(&mut self, ctx: &mut Context) -> Result<(), KioskError> {
        let _ = ctx;
        todo!(
            "build FontAtlas from {:?} at {}px — DESIGN.md §8",
            self.font_path,
            self.size_px
        )
    }

    fn update(&mut self, ctx: &mut Context, dt: f32) {
        let _ = (ctx, dt);
        todo!("re-render HH:MM:SS text when the second changes — DESIGN.md §8")
    }
}
