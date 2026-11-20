//! The frame's keys and mouse.
//!
//! The keys down (by scan code; the window's messages keep them) are copied for the frame,
//! with the mouse's buttons as keys 0x73 and 0x74 and two pairs of keys made one; a key is
//! pressed when it went down this frame, or by auto-repeat: held 0x41 frames, it is pressed
//! again each frame.

/// Keys in a table.
pub const KEYS: usize = 0x80;
/// The mouse's buttons as keys.
pub(crate) const LEFT_BUTTON_KEY: usize = 0x73;
pub(crate) const RIGHT_BUTTON_KEY: usize = 0x74;
/// Pairs of keys made one: the two controls, the two shifts.
pub const PAIRS: [(usize, usize); 2] = [(0x1D, 0x61), (0x2A, 0x36)];
/// The screenshot key, looked at in the frame's copy.
pub const SCREENSHOT_KEY: usize = 0x52;

/// The mouse word's parts: the buttons (low byte), x (bits 8 .. 19) and y (bits 20 .. 31),
/// both signed.
pub fn unpack_mouse(word: u32) -> (u32, i32, i32) {
    (
        word & 0xFF,
        ((word << 12) as i32) >> 20,
        (word as i32) >> 20,
    )
}

/// The buttons as the game keeps them: all, the left (bit 0), the right (bit 1).
pub fn buttons(buttons: u32) -> [u32; 3] {
    [buttons, buttons & 1, (buttons >> 1) & 1]
}

/// The frame's key tables: now, before, pressed and repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyTables {
    pub now: [u8; KEYS],
    pub before: [u8; KEYS],
    pub pressed: [u8; KEYS],
    pub repeat: [u8; KEYS],
}

impl KeyTables {
    /// The keys down (the window's table) copied for the frame.
    pub fn copy_keys(&mut self, down: &[u8; KEYS]) {
        self.now = *down;
    }

    /// Each pair of keys made one: both the `or` of the two.
    pub fn join_pairs(&mut self) {
        for (a, b) in PAIRS {
            let both = self.now[a] | self.now[b];
            self.now[a] = both;
            self.now[b] = both;
        }
    }

    /// The buttons' low bytes as their keys.
    pub fn buttons_to_keys(&mut self, left: u32, right: u32) {
        self.now[LEFT_BUTTON_KEY] = left as u8;
        self.now[RIGHT_BUTTON_KEY] = right as u8;
    }

    /// The keys down now and not before (their low bits only, a dword at a time), and
    /// before = now. The last dword's pressed and now.
    pub fn find_pressed(&mut self) -> (u32, u32) {
        let mut last = (0, 0);
        for at in (0..KEYS).step_by(4) {
            let dword =
                |t: &[u8; KEYS]| u32::from_le_bytes([t[at], t[at + 1], t[at + 2], t[at + 3]]);
            let (before, now) = (dword(&self.before), dword(&self.now));
            self.before[at..at + 4].copy_from_slice(&now.to_le_bytes());
            let pressed = !before & now & 0x0101_0101;
            self.pressed[at..at + 4].copy_from_slice(&pressed.to_le_bytes());
            last = (pressed, now);
        }
        last
    }

    /// For keys 1 .. 0x7F the frames held (0 when up); at 0x41 the key is pressed again
    /// and the count goes back to 0x40.
    pub fn repeat(&mut self) {
        for k in 1..KEYS {
            self.repeat[k] = if self.now[k] == 0 {
                0
            } else {
                self.repeat[k].wrapping_add(1)
            };
            if self.repeat[k] == 0x41 {
                self.pressed[k] = 1;
                self.repeat[k] = 0x40;
            }
        }
    }

    /// The left and right clicks: their keys pressed (low bits).
    pub fn clicks(&self) -> [u32; 2] {
        [
            self.pressed[LEFT_BUTTON_KEY] as u32 & 1,
            self.pressed[RIGHT_BUTTON_KEY] as u32 & 1,
        ]
    }
}

/// The dwords the end of a frame clears: the debug page and the race freeze.
pub(crate) const DEBUG_WORDS: usize = 2;

/// `input::end_frame`: with screenshots on, `check` looks at the screenshot key; the dwords as
/// left.
pub fn end_frame(screenshots_on: u32, check: impl FnOnce()) -> [u32; DEBUG_WORDS] {
    if screenshots_on != 0 {
        check();
    }
    [0; DEBUG_WORDS]
}

/// Whether a screenshot is wanted: the screenshot key is down in the frame's copy.
pub fn screenshot_wanted(key_now: u8) -> bool {
    key_now != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_and_keys() {
        assert_eq!(unpack_mouse(0xFFF8_0102), (2, -0x7FF, -1));
        assert_eq!(buttons(3), [3, 1, 1]);
        let mut t = KeyTables {
            now: [0; KEYS],
            before: [0; KEYS],
            pressed: [0; KEYS],
            repeat: [0; KEYS],
        };
        t.now[0x1D] = 1;
        t.now[5] = 1;
        t.before[6] = 1;
        t.now[6] = 1;
        t.repeat[6] = 0x40;
        let mut down = [0; KEYS];
        down[0x1D] = 1;
        down[5] = 1;
        down[6] = 1;
        t.copy_keys(&down);
        t.join_pairs();
        assert_eq!(t.now[0x61], 1);
        t.buttons_to_keys(1, 0);
        t.find_pressed();
        t.repeat();
        assert_eq!((t.pressed[5], t.pressed[6], t.repeat[6]), (1, 1, 0x40));
        assert_eq!(t.before[5], 1);
        assert_eq!(t.clicks(), [1, 0]);
    }

    #[test]
    fn frame_end() {
        let mut checked = false;
        assert_eq!(end_frame(0, || checked = true), [0, 0]);
        assert!(!checked);
        end_frame(1, || checked = true);
        assert!(checked);
        assert!(!screenshot_wanted(0) && screenshot_wanted(2));
    }
}
