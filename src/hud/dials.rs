//! The HUD's dials' set-ups (`hud::disc_*`, `hud::needle_*`): each copies fields of the
//! record at ebp and some variables of its own into the shape's variables (one copy only
//! when `dword_6D1502` is 0) and draws the shape (`hud::draw_disc`, `hud::draw_needle`);
//! the copies by `push` / `pop`, so no register changes before the call. The layouts are
//! the HUD's three (the record's +0xF4, `sub_4B1CD0`).

use crate::framework::rva;
use crate::hud::store::{HUD, Hud};
use crate::session::Cell32;
use crate::store::game_read;

/// Where a copy reads.
#[derive(Clone, Copy)]
pub enum From {
    /// A field of the HUD's record.
    Field(u32),
    /// A variable of the HUD's block.
    Var(Cell),
}

/// A variable of the HUD's block.
pub type Cell = fn(&Hud) -> &dyn Cell32;

/// One set-up: its copies (from, to, only when `dword_6D1502` is 0).
pub struct Setup {
    pub copies: &'static [(From, Cell, bool)],
}

const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502

/// The Rust version of a set-up's copies (see the header); its caller then draws the
/// shape (`hud::draw_disc` or `hud::draw_needle`).
pub unsafe fn set_up(record: u32, s: &Setup) {
    unsafe {
        let flat = game_read::<u32>(FLAT_RVA) == 0;
        for &(from, to, only_flat) in s.copies {
            if only_flat && !flat {
                continue;
            }
            let v = match from {
                From::Field(o) => ((record + o) as *const u32).read_unaligned(),
                From::Var(c) => c(&HUD).bits(),
            };
            to(&HUD).set_bits(v);
        }
    }
}

/// A disc's set-up and the disc.
pub unsafe fn disc(record: u32, s: &Setup) -> Result<(), crate::errors::Refused> {
    unsafe {
        set_up(record, s);
        crate::hud::draw_disc::draw_disc()
    }
}

/// A needle's set-up and the needle.
pub unsafe fn needle(record: u32, s: &Setup) -> Result<(), crate::errors::Refused> {
    unsafe {
        set_up(record, s);
        crate::hud::draw_needle::draw_needle()
    }
}
