//! Per-frame framework context handed to every [`crate::KioskApp`] method.

use wilhelm_renderer::core::{Camera2D, CameraController, Renderer};
use wilhelm_renderer::graphics2d::shapes::ShapeRenderable;

/// Stable handle to a shape owned by the framework's shape store.
///
/// Unlike the raw stack — where `App::run` re-sorts its shape `Vec` in
/// place every frame, so slice indices silently move — a `ShapeId` stays
/// valid until the shape is removed, regardless of z-order changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeId(u64);

/// Framework services available to the application: the renderer, the shape
/// store, the optional camera, frame statistics, and exit control.
///
/// Handle-based by design (see `docs/DESIGN.md` §4-5): applications refer
/// to shapes through [`ShapeId`], never through indices or references into
/// framework storage.
pub struct Context {
    renderer: Renderer,
    shapes: Vec<(ShapeId, ShapeRenderable)>,
    next_shape_id: u64,
    camera_ctrl: Option<CameraController>,
    size: (i32, i32),
    fps: FpsCounter,
    exit_requested: bool,
}

// The pub(crate) plumbing below is consumed by the frame loop
// (`Kiosk::run`), which is still a stub; drop this allow when the loop
// lands.
#[allow(dead_code)]
impl Context {
    pub(crate) fn new(
        renderer: Renderer,
        size: (i32, i32),
        camera_ctrl: Option<CameraController>,
    ) -> Self {
        Self {
            renderer,
            shapes: Vec::new(),
            next_shape_id: 0,
            camera_ctrl,
            size,
            fps: FpsCounter::new(),
            exit_requested: false,
        }
    }

    /// The 2D renderer, for direct drawing (text runs, meshes) outside the
    /// managed shape store.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Current framebuffer size in pixels (live: tracks resize events,
    /// though under cage the size is fixed at the monitor's mode).
    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// Add a shape to the managed store. Shapes render every frame in
    /// z-order. The returned handle stays valid until [`Self::remove_shape`].
    pub fn add_shape(&mut self, shape: ShapeRenderable) -> ShapeId {
        let id = ShapeId(self.next_shape_id);
        self.next_shape_id += 1;
        self.shapes.push((id, shape));
        id
    }

    /// Mutable access to a stored shape (position, style, z-order, …).
    pub fn shape_mut(&mut self, id: ShapeId) -> Option<&mut ShapeRenderable> {
        self.shapes
            .iter_mut()
            .find(|(sid, _)| *sid == id)
            .map(|(_, shape)| shape)
    }

    /// Remove and return a stored shape.
    pub fn remove_shape(&mut self, id: ShapeId) -> Option<ShapeRenderable> {
        let index = self.shapes.iter().position(|(sid, _)| *sid == id)?;
        Some(self.shapes.remove(index).1)
    }

    /// The world camera, if one was configured via
    /// [`crate::Kiosk::with_camera`].
    pub fn camera(&self) -> Option<&Camera2D> {
        self.camera_ctrl.as_ref().map(|ctrl| ctrl.camera())
    }

    /// Exponentially-weighted average frames per second.
    pub fn fps(&self) -> f32 {
        self.fps.value()
    }

    /// Seconds since renderer initialization (monotonic; wraps
    /// `Renderer::get_time`).
    pub fn time(&self) -> f64 {
        self.renderer.get_time()
    }

    /// Ask the framework to exit cleanly after the current frame:
    /// [`crate::KioskApp::shutdown`] runs, teardown is ordered, and
    /// [`crate::Kiosk::run`] returns `Ok`.
    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    pub(crate) fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    pub(crate) fn set_size(&mut self, size: (i32, i32)) {
        self.size = size;
    }

    /// Render all stored shapes in z-order without disturbing insertion
    /// order (a sorted index is rebuilt instead of sorting the store).
    pub(crate) fn render_shapes(&mut self) {
        todo!("z-ordered render pass over the shape store — DESIGN.md §5")
    }

    pub(crate) fn tick_fps(&mut self) {
        let now = self.renderer.get_time();
        self.fps.tick(now);
    }
}

/// Exponentially-weighted moving-average FPS counter (formula matching the
/// one production apps hand-roll today; see DESIGN.md §5).
#[cfg_attr(not(test), allow(dead_code))] // driven by the frame loop, still a stub
struct FpsCounter {
    last_time: Option<f64>,
    ewma_frame_s: f32,
}

impl FpsCounter {
    const ALPHA: f32 = 0.1;

    fn new() -> Self {
        Self {
            last_time: None,
            ewma_frame_s: 0.0,
        }
    }

    fn tick(&mut self, now: f64) {
        if let Some(last) = self.last_time {
            let frame_s = (now - last) as f32;
            if self.ewma_frame_s == 0.0 {
                self.ewma_frame_s = frame_s;
            } else {
                self.ewma_frame_s += Self::ALPHA * (frame_s - self.ewma_frame_s);
            }
        }
        self.last_time = Some(now);
    }

    fn value(&self) -> f32 {
        if self.ewma_frame_s > 0.0 {
            1.0 / self.ewma_frame_s
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FpsCounter;

    #[test]
    fn fps_counter_converges_to_frame_rate() {
        let mut fps = FpsCounter::new();
        for i in 0..200 {
            fps.tick(i as f64 * (1.0 / 60.0));
        }
        assert!((fps.value() - 60.0).abs() < 1.0);
    }

    #[test]
    fn fps_counter_reports_zero_before_two_frames() {
        let mut fps = FpsCounter::new();
        assert_eq!(fps.value(), 0.0);
        fps.tick(0.0);
        assert_eq!(fps.value(), 0.0);
    }
}
