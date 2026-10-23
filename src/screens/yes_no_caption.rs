//! `screens::yes_no_caption` (0x46FA5A): the "leave?" question's button shows the
//! answer.
//!
//! In:  nothing
//! Out: eax as `strings::append_string_by_id` leaves it, ebx = after the NUL
//!
//! String 0x56B + `g_yesNoAnswer` (0x56B "não", 0x56C "sim", by the language file) and
//! a NUL into `g_captionText`, from which `screens::draw_highlighted_caption` draws the
//! button. In the question screen's draw list (0x758C9E). IDA has not made it a
//! function.
//! Refuses a caption longer than its place.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};
use crate::strings::append_string_by_id::append_string_by_id;

// ---- game data ----

const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const CAPTION_LEN: usize = 0x100;
const FIRST: u32 = 0x56B;

// ---- Rust version ----

/// Returns eax and ebx as the original leaves them.
pub unsafe fn yes_no_caption() -> Result<(u32, u32), Refused> {
    unsafe {
        let id = game_read::<u32>(ANSWER_RVA).wrapping_add(FIRST);
        let mut text = Vec::new();
        let eax = append_string_by_id(id, &mut |p, n| {
            text.extend_from_slice(std::slice::from_raw_parts(p, n))
        });
        // Converted: `rc2k_core::screens::captions::fit`.
        let text = rc2k_core::screens::captions::fit(text, CAPTION_LEN)?;
        let caption = game_addr(CAPTION_RVA) as *mut u8;
        std::ptr::copy_nonoverlapping(text.as_ptr(), caption, text.len());
        Ok((eax, caption as u32 + text.len() as u32))
    }
}

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    let (_eax, _ebx) = unsafe { yes_no_caption() }?;
    Ok(())
}
