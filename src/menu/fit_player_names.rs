//! `menu::fit_player_names` (0x411445): cuts each player's name until it is no wider than
//! 350 pixels.
//!
//! In:  ecx (the x the first measuring is done with)
//! Out: every register preserved
//! For each of the four names in `g_playerNames` (`menu::fit_name`, 0x411474, a plain
//! function here): the name is measured with the font at 0x51E824 (`text::measure_text`)
//! and, while it is wider than 0x15E, loses its last character and is measured again.
//! The x given to a measuring is the width the one before gave (the first gets the
//! caller's ecx): the original leaves it in ecx, kept.
//! One caller, the main menu's set-up `menu::set_up_front_end`.
//! Refuses a name that would be cut to before its start, and what the measuring refuses.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_ptr, rva};
use crate::menu::player_names::{PLAYERS, name};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::MEASURING_FONT;
use crate::text::measure_text::measure_text_at;
use rc2k_core::menu::text_entry;

// ---- game data ----

/// More characters than a name is looked through for its end.
const LONGEST: usize = 0x1000;

// ---- Rust version ----

/// `menu::fit_name` (0x411474). Returns the width of the name as it is left.
pub unsafe fn fit_name(text: *mut u8, x: u32) -> Result<u32, Refused> {
    unsafe {
        let Some(len) = (0..LONGEST).find(|&n| text.add(n).read() == 0) else {
            return Err(BadInput("a name without its end").into());
        };
        // Converted: `rc2k_core::menu::text_entry::fit_name`.
        let font: *mut Font = game_ptr(rva(MEASURING_FONT));
        text_entry::fit_name(len, x, |len, x| -> Result<u32, Refused> {
            text.add(len).write(0);
            Ok(measure_text_at(font, text, x as i32, &mut GameCanvas)?.width)
        })
    }
}

pub unsafe fn fit_player_names(x: u32) -> Result<(), Refused> {
    let mut x = x;
    for player in 0..PLAYERS {
        x = unsafe { fit_name(name(player), x) }?;
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fit_player_names(r.ecx) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test: names of the test's in the game's place, measured with the game's
// font; everything put back ----
