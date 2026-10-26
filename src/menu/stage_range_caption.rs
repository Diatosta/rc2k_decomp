//! `menu::stage_range_caption` (0x4A1FBB): "first-last" into `g_captionText`: the first
//! (`dword_7556B8`) and it plus 9 (a reading: the stages shown on a page).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event pages' code (`sub_4A1F7A`, ..). `strings::append_u32`'s Rust
//! version; the minus and the NUL written in place.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const FIRST_RVA: usize = rva(0x7556B8); // dword_7556B8
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn stage_range_caption(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(FIRST_RVA) };
    // Converted: `rc2k_core::menu::event::stage_range`.
    let mut text = rc2k_core::menu::event::stage_range(first);
    text.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(CAPTION_RVA) as *mut u8, text.len())
    };
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { stage_range_caption(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
