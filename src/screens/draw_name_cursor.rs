//! `screens::draw_name_cursor` (0x467D6C): the cursor after a player's name being typed.
//!
//! In:  edi = the player's player (`Player`)
//! Out: every register preserved; carry of the last text
//!
//! The name (`Player::name`) measured with the font at 0x51E824 (its line count and widest
//! line are left in it); not drawn, the carry is that of `cmp dword_7559A8, 1`; with the
//! text cursor's blink (`dword_7559A8`) at 1, string 4
//! ("_") at x = 0x1BB + the name's width, y = 0xD4, in the font at 0x51E8C4. Three
//! callers (the player details screens).
//! Refuses no player or no name.
//! Self-test: the four players' names, both blinks, nothing drawn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::menu::player::Player;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- game data ----

pub const BLINK_RVA: usize = rva(0x7559A8); // dword_7559A8
// ---- Rust version ----

/// `x` is ecx at entry (the measuring's x). Returns the carry.
pub unsafe fn draw_name_cursor(
    player: u32,
    x: u32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    if player == 0 {
        return Err(BadInput("no player object").into());
    }
    unsafe {
        let name = (player as *const Player).read_unaligned().name;
        if name == 0 {
            return Err(BadInput("a player without its name").into());
        }
        // Converted: `rc2k_core::screens::player_details::name_cursor`.
        rc2k_core::screens::player_details::name_cursor(
            &mut crate::screens::game_screen::DetailsScreen::new(canvas),
            name,
            x as i32,
            game_read(BLINK_RVA),
        )
    }
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe {
        draw_name_cursor(r.edi, r.ecx, canvas)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_name_cursor", r, &rust)
}
