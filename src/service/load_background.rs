//! `service::load_background` (0x470F55): the service area's picture for the current
//! player's car: the player (`g_players`[`dword_75598C`]), its car (+8; car 0x21 takes car
//! 0x17's picture), the picture `off_75A52E`[car] (`files\g\graphics\screens\servNN.png`, 60
//! of them) loaded (`gfx::load_picture`) and converted (`gfx::convert_picture`); a failed
//! conversion (carry) is fatal error 0x80712435.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the service area's screens (0x47296A, 0x472B53, 0x4A1C0F). Its calls through
//! `part` (0 the loading with eax the path, 1 the conversion; in the game their Rust
//! versions). Refuses (before anything) a current player past the four or missing, and a car
//! past the table.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::menu::parts::Part;

const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // dword_75598C
const PICTURES_RVA: usize = rva(0x75A52E); // off_75A52E: the service pictures' paths
// gfx::load_picture, gfx::convert_picture
const NOT_CONVERTED: FatalError = FatalError {
    code: 0x8071_2435,
    message: "",
};

/// The Rust version: every register kept.
pub unsafe fn load_background(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let player = crate::menu::player::player_at(unsafe { game_read(CURRENT_PLAYER_RVA) });
    if player == 0 {
        return Err(BadInput("no current player (or one past the four)").into());
    }
    // Converted: `rc2k_core::service::car::background`.
    let car = rc2k_core::service::car::background(unsafe {
        ((player + 8) as *const u32).read_unaligned()
    })
    .map_err(BadInput::from)?;
    let mut r = Regs {
        eax: unsafe { game_read(PICTURES_RVA + 4 * car as usize) },
        ebx: car,
        edi: player,
        ..*entry
    };
    part(0, &mut r)?;
    part(1, &mut r)?;
    if r.carry() {
        return Err(NOT_CONVERTED.into());
    }
    Ok(*entry)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => {
                crate::gfx::load_picture::load_picture(r.eax as *mut u8)?;
            }
            _ => {
                crate::gfx::convert_picture::convert_picture_now()?;
                r.set_carry(false);
            }
        }
    }
    Ok(())
}
