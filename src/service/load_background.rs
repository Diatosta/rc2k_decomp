//! `service::load_background` (0x470F55): the service area's picture for the current
//! player's car: the player (`g_players`[`dword_75598C`]), its car (+8; car 0x21 takes car
//! 0x17's picture), the picture `off_75A52E`[car] (`files\g\graphics\screens\servNN.png`, 60
//! of them) loaded (`gfx::load_picture`) and converted (`gfx::convert_picture`); a failed
//! conversion (carry) is fatal error 0x80712435.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the service area's screens (0x47296A, 0x472B53, 0x4A1C0F). Its calls: the
//! Rust versions. Refuses (before anything) a current player past the four or missing, and a car
//! past the table.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::menu::lists_store::MENU_LISTS;

const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // dword_75598C

/// The Rust version (see the header). The conversion's carry (fatal error 0x80712435) is
/// never set by `gfx::convert_picture`'s Rust version, which refuses instead.
pub unsafe fn load_background() -> Result<(), Refused> {
    let player = crate::menu::player::player_at(unsafe { game_read(CURRENT_PLAYER_RVA) });
    if player == 0 {
        return Err(BadInput("no current player (or one past the four)").into());
    }
    // Converted: `rc2k_core::service::car::background`.
    let car = rc2k_core::service::car::background(unsafe {
        ((player + 8) as *const u32).read_unaligned()
    })
    .map_err(BadInput::from)?;
    unsafe {
        let path = MENU_LISTS.service_pictures[car as usize].get();
        crate::gfx::load_picture::load_picture(path as *mut u8)?;
        crate::gfx::convert_picture::convert_picture_now()?;
    }
    Ok(())
}
