//! `menu::begin_player_name` (0x467C34): typing a player's name begins.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Unless the Enter came with ah = 4 (`dword_7554D4` at 1): the current player's name
//! kept (`menu/player_name.rs`), cleared (`menu::clear_text`), and the entry going on
//! (`dword_7556A8` = 1); then, either way, the length typed (`dword_75556D`) 0. The
//! action of the name's item of the four players' details pages.
//! Refuses no current player or a player without a name.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::menu::player_name::{kept, name};
use crate::menu::text_entry::{Text, field, put_field, with_entry};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const ENTER_WITH_4_RVA: usize = rva(0x7554D4); // dword_7554D4

// ---- Rust version ----

pub unsafe fn begin_player_name() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::text_entry::Entry::begin_player_name`.
    let with_4 = unsafe { game_read::<u32>(ENTER_WITH_4_RVA) } == 1;
    let at = if with_4 { None } else { Some(name()?) };
    unsafe {
        // The copy is written, not read.
        let (mut text, mut copy) = (at.map_or(Text::default(), |at| field(at)), Text::default());
        with_entry(|e| e.begin_player_name(with_4, &mut text, &mut copy));
        if let Some(at) = at {
            put_field(at, &text);
            put_field(kept(), &copy);
        }
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_player_name() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
