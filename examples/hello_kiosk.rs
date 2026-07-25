//! The WilhelmOS reference demo, rewritten against wilhelmos_kiosk.
//!
//! Compare with `wilhelmos-kiosk-demo/src/main.rs` (the raw-stack
//! original): same behavior — a movable, scalable triangle with an ImGui
//! control panel — but state lives in plain struct fields and there is not
//! a single `Rc<RefCell<..>>` in sight. On top of the original: an FPS
//! overlay (composed framework component) and Escape to exit.

use wilhelmos_kiosk::{
    Color, Context, Event, FpsOverlay, Key, Kiosk, KioskApp, KioskError, ShapeId, ShapeKind,
    ShapeRenderable, ShapeStyle, Ui,
};
use wilhelm_renderer::graphics2d::shapes::Triangle;

#[derive(Default)]
struct HelloApp {
    triangle: Option<ShapeId>,
    pos: (f32, f32),
    scale: f32,
    size: (f32, f32),
    fps: FpsOverlay,
}

impl KioskApp for HelloApp {
    fn init(&mut self, ctx: &mut Context) -> Result<(), KioskError> {
        let (w, h) = ctx.size();
        self.size = (w as f32, h as f32);
        self.pos = (self.size.0 / 2.0, self.size.1 / 2.0);
        self.scale = 1.0;

        // Size everything relative to the display, exactly like the
        // original demo.
        let half = self.size.1 * 0.15;
        let triangle = ShapeRenderable::from_shape(
            ShapeKind::Triangle(Triangle::new([
                (-half, half * 0.5),
                (half, half * 0.5),
                (0.0, -half),
            ])),
            ShapeStyle::fill(Color::from_rgb(0.2, 0.6, 0.9)),
        );
        self.triangle = Some(ctx.add_shape(triangle));
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context, _dt: f32) {
        if let Some(id) = self.triangle {
            if let Some(shape) = ctx.shape_mut(id) {
                shape.set_position(self.pos.0, self.pos.1);
                shape.set_scale(self.scale);
            }
        }
    }

    fn ui(&mut self, ui: &Ui<'_>, ctx: &mut Context) {
        ui.window("Shape Controls", 0, |im| {
            im.text("Position");
            im.slider_float("X", &mut self.pos.0, 0.0, self.size.0);
            im.slider_float("Y", &mut self.pos.1, 0.0, self.size.1);
            im.separator();
            im.text("Transform");
            im.slider_float("Scale", &mut self.scale, 0.1, 3.0);
        });
        self.fps.ui(ui, ctx);
    }

    fn on_event(&mut self, event: &Event, ctx: &mut Context) {
        if let Event::Key {
            key: Key::ESCAPE,
            action,
            ..
        } = event
        {
            if action.is_press() {
                ctx.request_exit();
            }
        }
    }
}

fn main() -> Result<(), KioskError> {
    Kiosk::new("Hello Kiosk")
        .background(Color::from_rgb(0.1, 0.1, 0.15))
        .run(HelloApp::default())
}
