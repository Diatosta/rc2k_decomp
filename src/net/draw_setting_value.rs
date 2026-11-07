//! `net::draw_setting_value` (0x416BC4): a setting's value drawn (`text::draw_text` with
//! `font` at 0x190, y ebp; ebp 0x1E lower): for the item being edited (`dword_52A4B8`) the
//! text with string 4 (the cursor) after it, built in the game's text buffer; an empty text
//! as string 0x16E; any other as it is.
//!
//! In:  eax = the text, esi = the item, ebp = y
//! Out: as `text::draw_text` leaves them (eax 0, ebx past the text, ecx, edx); ebp + 0x1E;
//!      no `pusha`
//! Callers: `net::settings_frame`, `net::draw_number_value` (which goes on into it).

use crate::errors::Refused;
use crate::net::lobby::{FONT, push_id, store_text};
use crate::net::store::NetVars;
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::screens::{self, Value};

/// The Rust version: the text `text` (up to its NUL) for item `item` at `y` (x 0x190),
/// edited (with the caret), nothing (the "nothing" string) or as it is; the first two are
/// built in the game's text buffer. Returns the next y (y + 0x1E, the original's ebp). The
/// registers it leaves are read by no caller (`settings_frame`, `draw_number_value`).
pub fn draw_setting_value(vars: &NetVars, text: &[u8], item: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`text_value`, `Value`, `CARET`, `NOTHING`).
    let value = screens::text_value(item == vars.edited.get(), || {
        text.first().copied().unwrap_or(0)
    });
    let shown = match value {
        Value::Edited => {
            let mut built = text.to_vec();
            push_id(&mut built, screens::CARET)?;
            store_text(built)?
        }
        Value::Nothing => {
            let mut built = Vec::new();
            push_id(&mut built, screens::NOTHING)?;
            store_text(built)?
        }
        Value::AsIs => {
            let mut plain = text.to_vec();
            plain.push(0);
            plain
        }
    };
    crate::text::draw_text::draw_text_at(FONT, &shown, 0x190, y as i32, &mut GameCanvas)?;
    Ok(y.wrapping_add(0x1E))
}
