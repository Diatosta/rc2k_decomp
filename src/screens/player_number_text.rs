//! `screens::player_number_text` (0x46A359): the current player's title, as a text of the
//! car choice.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! String 0x191 + the current player's number (`dword_75598C`), a space, string 0x170, a NUL, into
//! `byte_A8C054`. In six entry and draw lists (the car choice's). Refuses a text longer than its
//! place (0x100). The menus and draw lists run its `entry` (`menu::table_calls`); the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::menu::event_store::EVENT_STATE;
use crate::strings::append_string_by_id::append_string_by_id;

const TEXT_RVA: usize = rva(0xA8C054); // byte_A8C054
const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x191;
const SECOND_STRING: u32 = 0x170;

pub unsafe fn player_number_text() -> Result<(), Refused> {
    unsafe {
        let n: u32 = EVENT_STATE.current_player.get();
        let mut text = Vec::new();
        let mut emit =
            |p: *const u8, len: usize| text.extend_from_slice(std::slice::from_raw_parts(p, len));
        append_string_by_id(FIRST_STRING.wrapping_add(n), &mut emit);
        text.push(b' ');
        let mut emit =
            |p: *const u8, len: usize| text.extend_from_slice(std::slice::from_raw_parts(p, len));
        append_string_by_id(SECOND_STRING, &mut emit);
        // Converted: `rc2k_core::screens::captions::fit`.
        let text = rc2k_core::screens::captions::fit(text, TEXT_LEN)?;
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(TEXT_RVA) as *mut u8, text.len());
    }
    Ok(())
}
