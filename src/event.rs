//! Typed input events over the raw GLFW callback arguments.
//!
//! The raw stack hands applications five separate `i32`-typed callbacks;
//! this module folds them into one [`Event`] enum with newtyped codes and
//! named constants, so application code reads `Key::O` instead of a magic
//! `79`.
//!
//! The newtypes deliberately wrap the raw `i32` (rather than being Rust
//! enums) so that unknown codes pass through losslessly and new GLFW keys
//! never become a breaking change. Named constants reuse the renderer's
//! re-exported GLFW constants where they exist; printable keys (letters,
//! digits), which the renderer does not export, use GLFW's stable ASCII
//! values directly.

use wilhelm_renderer::core::engine::glfw as g;

/// A keyboard key code (GLFW key space).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(pub i32);

impl Key {
    pub const SPACE: Key = Key(g::GLFW_KEY_SPACE);
    pub const ESCAPE: Key = Key(g::GLFW_KEY_ESCAPE);
    pub const ENTER: Key = Key(g::GLFW_KEY_ENTER);
    pub const TAB: Key = Key(g::GLFW_KEY_TAB);
    pub const BACKSPACE: Key = Key(g::GLFW_KEY_BACKSPACE);
    pub const INSERT: Key = Key(g::GLFW_KEY_INSERT);
    pub const DELETE: Key = Key(g::GLFW_KEY_DELETE);
    pub const RIGHT: Key = Key(g::GLFW_KEY_RIGHT);
    pub const LEFT: Key = Key(g::GLFW_KEY_LEFT);
    pub const DOWN: Key = Key(g::GLFW_KEY_DOWN);
    pub const UP: Key = Key(g::GLFW_KEY_UP);
    pub const PAGE_UP: Key = Key(g::GLFW_KEY_PAGE_UP);
    pub const PAGE_DOWN: Key = Key(g::GLFW_KEY_PAGE_DOWN);
    pub const HOME: Key = Key(g::GLFW_KEY_HOME);
    pub const END: Key = Key(g::GLFW_KEY_END);
    pub const F1: Key = Key(g::GLFW_KEY_F1);
    pub const F2: Key = Key(g::GLFW_KEY_F2);
    pub const F3: Key = Key(g::GLFW_KEY_F3);
    pub const F4: Key = Key(g::GLFW_KEY_F4);
    pub const F5: Key = Key(g::GLFW_KEY_F5);
    pub const F6: Key = Key(g::GLFW_KEY_F6);
    pub const F7: Key = Key(g::GLFW_KEY_F7);
    pub const F8: Key = Key(g::GLFW_KEY_F8);
    pub const F9: Key = Key(g::GLFW_KEY_F9);
    pub const F10: Key = Key(g::GLFW_KEY_F10);
    pub const F11: Key = Key(g::GLFW_KEY_F11);
    pub const F12: Key = Key(g::GLFW_KEY_F12);
    pub const LEFT_SHIFT: Key = Key(g::GLFW_KEY_LEFT_SHIFT);
    pub const LEFT_CONTROL: Key = Key(g::GLFW_KEY_LEFT_CONTROL);
    pub const LEFT_ALT: Key = Key(g::GLFW_KEY_LEFT_ALT);
    pub const LEFT_SUPER: Key = Key(g::GLFW_KEY_LEFT_SUPER);
    pub const RIGHT_SHIFT: Key = Key(g::GLFW_KEY_RIGHT_SHIFT);
    pub const RIGHT_CONTROL: Key = Key(g::GLFW_KEY_RIGHT_CONTROL);
    pub const RIGHT_ALT: Key = Key(g::GLFW_KEY_RIGHT_ALT);
    pub const RIGHT_SUPER: Key = Key(g::GLFW_KEY_RIGHT_SUPER);

    // Printable keys — GLFW uses the ASCII value; not exported by the
    // renderer, defined here from the stable GLFW ABI.
    pub const NUM_0: Key = Key(48);
    pub const NUM_1: Key = Key(49);
    pub const NUM_2: Key = Key(50);
    pub const NUM_3: Key = Key(51);
    pub const NUM_4: Key = Key(52);
    pub const NUM_5: Key = Key(53);
    pub const NUM_6: Key = Key(54);
    pub const NUM_7: Key = Key(55);
    pub const NUM_8: Key = Key(56);
    pub const NUM_9: Key = Key(57);
    pub const A: Key = Key(65);
    pub const B: Key = Key(66);
    pub const C: Key = Key(67);
    pub const D: Key = Key(68);
    pub const E: Key = Key(69);
    pub const F: Key = Key(70);
    pub const G: Key = Key(71);
    pub const H: Key = Key(72);
    pub const I: Key = Key(73);
    pub const J: Key = Key(74);
    pub const K: Key = Key(75);
    pub const L: Key = Key(76);
    pub const M: Key = Key(77);
    pub const N: Key = Key(78);
    pub const O: Key = Key(79);
    pub const P: Key = Key(80);
    pub const Q: Key = Key(81);
    pub const R: Key = Key(82);
    pub const S: Key = Key(83);
    pub const T: Key = Key(84);
    pub const U: Key = Key(85);
    pub const V: Key = Key(86);
    pub const W: Key = Key(87);
    pub const X: Key = Key(88);
    pub const Y: Key = Key(89);
    pub const Z: Key = Key(90);
}

/// A mouse button code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MouseButton(pub i32);

impl MouseButton {
    pub const LEFT: MouseButton = MouseButton(g::GLFW_MOUSE_BUTTON_LEFT);
    pub const RIGHT: MouseButton = MouseButton(g::GLFW_MOUSE_BUTTON_RIGHT);
    pub const MIDDLE: MouseButton = MouseButton(g::GLFW_MOUSE_BUTTON_MIDDLE);
}

/// A key/button action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Action(pub i32);

impl Action {
    pub const RELEASE: Action = Action(g::GLFW_RELEASE);
    pub const PRESS: Action = Action(g::GLFW_PRESS);
    pub const REPEAT: Action = Action(g::GLFW_REPEAT);

    pub fn is_press(self) -> bool {
        self == Action::PRESS
    }

    pub fn is_release(self) -> bool {
        self == Action::RELEASE
    }

    pub fn is_repeat(self) -> bool {
        self == Action::REPEAT
    }
}

/// Modifier-key bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mods(pub i32);

impl Mods {
    pub fn shift(self) -> bool {
        self.0 & g::GLFW_MOD_SHIFT != 0
    }

    pub fn ctrl(self) -> bool {
        self.0 & g::GLFW_MOD_CONTROL != 0
    }

    pub fn alt(self) -> bool {
        self.0 & g::GLFW_MOD_ALT != 0
    }

    pub fn superkey(self) -> bool {
        self.0 & g::GLFW_MOD_SUPER != 0
    }

    pub fn caps_lock(self) -> bool {
        self.0 & g::GLFW_MOD_CAPS_LOCK != 0
    }

    pub fn num_lock(self) -> bool {
        self.0 & g::GLFW_MOD_NUM_LOCK != 0
    }
}

/// A single input event, delivered to [`crate::KioskApp::on_event`] already
/// capture-filtered against ImGui (see `docs/DESIGN.md` §6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Key {
        key: Key,
        action: Action,
        mods: Mods,
    },
    MouseButton {
        button: MouseButton,
        action: Action,
        mods: Mods,
    },
    /// Cursor position in pixels, origin top-left.
    CursorPos { x: f64, y: f64 },
    Scroll { x: f64, y: f64 },
    /// Framebuffer resize (fixed-size under cage, but delivered for
    /// completeness and for windowed development hosts).
    Resize { width: i32, height: i32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mods_bitfield() {
        let mods = Mods(g::GLFW_MOD_SHIFT | g::GLFW_MOD_CONTROL);
        assert!(mods.shift());
        assert!(mods.ctrl());
        assert!(!mods.alt());
        assert!(!mods.superkey());
    }

    #[test]
    fn printable_keys_use_glfw_ascii_values() {
        // GLFW defines printable keys at their ASCII uppercase values.
        assert_eq!(Key::A.0, 'A' as i32);
        assert_eq!(Key::Z.0, 'Z' as i32);
        assert_eq!(Key::NUM_0.0, '0' as i32);
        assert_eq!(Key::NUM_9.0, '9' as i32);
        assert_eq!(Key::O.0, 79); // sky_guard's former magic number
    }

    #[test]
    fn action_predicates() {
        assert!(Action::PRESS.is_press());
        assert!(Action::RELEASE.is_release());
        assert!(Action::REPEAT.is_repeat());
        assert!(!Action::PRESS.is_release());
    }
}
