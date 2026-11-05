//! The HUD's dials' set-ups (`hud::disc_*`, `hud::needle_*`): each copies fields of the
//! record at ebp and some variables of its own into the shape's variables (one copy only
//! when `dword_6D1502` is 0) and draws the shape (`hud::draw_disc`, `hud::draw_needle`);
//! the copies by `push` / `pop`, so no register changes before the call. The layouts are
//! the HUD's three (the record's +0xF4, `sub_4B1CD0`).

use crate::errors::Refused;
use crate::hud::store::{HUD, Hud};
use crate::pointers::GamePtr;
use crate::race::panel_store::PANEL;
use crate::session::Cell32;

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

/// The Rust version of a set-up's copies (see the header); its caller then draws the
/// shape (`hud::draw_disc` or `hud::draw_needle`).
pub fn set_up(record: GamePtr, s: &Setup) -> Result<(), Refused> {
    {
        let flat = PANEL.flat.get() == 0;
        for &(from, to, only_flat) in s.copies {
            if only_flat && !flat {
                continue;
            }
            let v = match from {
                // A panel field by the setup's offset (a table, as the original).
                // A record field by the setup's offset (a table, as the original).
                From::Field(o) => record.offset(o).read::<u32>("a hud record's field")?,
                From::Var(c) => c(&HUD).bits(),
            };
            to(&HUD).set_bits(v);
        }
    }
    Ok(())
}

/// A disc's set-up and the disc.
pub fn disc(record: GamePtr, s: &Setup) -> Result<(), crate::errors::Refused> {
    set_up(record, s)?;
    crate::hud::draw_disc::draw_disc()
}

/// A needle's set-up and the needle.
pub fn needle(record: GamePtr, s: &Setup) -> Result<(), crate::errors::Refused> {
    set_up(record, s)?;
    crate::hud::draw_needle::draw_needle()
}
