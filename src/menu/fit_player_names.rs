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

use crate::errors::{BadInput, Refused};
use crate::menu::player_names::{PLAYERS, name};
use crate::pointers::GamePtr;
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::MEASURING_FONT;
use crate::text::measure_text::measure_text_at;
use rc2k_core::menu::text_entry;
use rc2k_core::strings::bytes::Bytes;

// ---- game data ----

/// More characters than a name is looked through for its end.
const LONGEST: usize = 0x1000;

// ---- Rust version ----

/// `menu::fit_name` (0x411474). Returns the width of the name as it is left.
pub fn fit_name(text: u32, x: u32) -> Result<u32, Refused> {
    let name = GamePtr::new(text).bytes("a name")?;
    let Some(len) = (0..LONGEST).find(|&n| name.byte(n) == Some(0)) else {
        return Err(BadInput("a name without its end").into());
    };
    // Converted: `rc2k_core::menu::text_entry::fit_name`.
    let font: *mut Font = MEASURING_FONT.as_ptr();
    text_entry::fit_name(len, x, |len, x| -> Result<u32, Refused> {
        GamePtr::new(text + len as u32).write::<u8>(0, "a name")?;
        Ok(
            unsafe { measure_text_at(font, text as usize as *mut u8, x as i32, &mut GameCanvas) }?
                .width,
        )
    })
}

pub fn fit_player_names(x: u32) -> Result<(), Refused> {
    let mut x = x;
    for player in 0..PLAYERS {
        x = fit_name(name(player), x)?;
    }
    Ok(())
}
