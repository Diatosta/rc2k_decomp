//! The HUD's dials' set-ups (`hud::disc_*`, `hud::needle_*`): each copies fields of the
//! record at ebp and some variables of its own into the shape's variables (one copy only
//! when `dword_6D1502` is 0) and draws the shape (`hud::draw_disc`, `hud::draw_needle`);
//! the copies by `push` / `pop`, so no register changes before the call. The layouts are
//! the HUD's three (the record's +0xF4, `sub_4B1CD0`).

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

/// Where a copy reads.
#[derive(Clone, Copy)]
pub enum From {
    /// A field of the record at ebp.
    Field(u32),
    /// A variable (its RVA).
    Var(usize),
}

/// One set-up: its copies (from, to, only when `dword_6D1502` is 0) and, for the test,
/// the original's length, first bytes, call offset and callee.
pub struct Setup {
    pub copies: &'static [(From, usize, bool)],
}

const FLAT_RVA: usize = rva(0x6D1502); // dword_6D1502

/// A call by place: 1 the shape.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version of a set-up (see the header): the registers the shape leaves.
pub unsafe fn set_up(entry: &Regs, s: &Setup, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let flat = game_read::<u32>(FLAT_RVA) == 0;
        for &(from, to, only_flat) in s.copies {
            if only_flat && !flat {
                continue;
            }
            let v = match from {
                From::Field(o) => ((entry.ebp + o) as *const u32).read_unaligned(),
                From::Var(r) => game_read(r),
            };
            game_write(to, v);
        }
    }
    let mut g = *entry;
    part(1, &mut g)?;
    Ok(g)
}
