//! The name plates' tests' stand-ins (see `race/name_plates.rs`): the lock hands out a test
//! surface (or fails), the text and the rest note themselves.

use crate::framework::{Regs, game_write, rva};
use crate::race::level_init::note;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub static SURFACE: AtomicU32 = AtomicU32::new(0);
pub static FAIL: AtomicBool = AtomicBool::new(false);
/// The test surface: 0x40 rows of 0x40 dwords, a gap of 8 bytes.
pub const ROW_DWORDS: u32 = 0x40;
pub const GAP: u32 = 8;
pub const SIZE: usize = 0x40 * (0x100 + 8);

pub fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    if place == 3 {
        // The text's y in the note too (its eax slot is the font, which is fixed).
        let mut y = Regs { eax: r.edx, ..*r };
        note(9, &mut y);
    }
    if place != 1 {
        return;
    }
    let fail = FAIL.load(Ordering::SeqCst);
    r.set_carry(fail);
    if !fail {
        unsafe {
            game_write(rva(0x4F49C0), SURFACE.load(Ordering::SeqCst));
            game_write(rva(0x4F49BC), ROW_DWORDS * 4 + GAP);
            game_write(rva(0x4F49B4), ROW_DWORDS);
            game_write(rva(0x4F49C4), GAP);
        }
    }
}
