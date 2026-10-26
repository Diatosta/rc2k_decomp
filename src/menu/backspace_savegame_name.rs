//! `menu::backspace_savegame_name` (0x46D5FD): takes the last character off the name of the
//! highlighted saved game: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! In:  nothing
//! Out: every register preserved
//! Also called by the replay name editor, which is a slip in the original (see
//! `edit_replay_name`). Refuses a name past the list and a length outside 0 .. 0x1F.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::menu::name_lists::{NAME_LEN, savegame_name};
use crate::menu::text_entry::{entry as text_entry, length, put_entry};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn backspace_savegame_name() -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::text_entry::Entry::backspace`.
    if length() == 0 {
        return Ok(());
    }
    let name = savegame_name()?;
    let mut e = text_entry();
    if let Some(at) = e.backspace(NAME_LEN)? {
        put_entry(&e);
        unsafe { name.add(at).write(0) };
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { backspace_savegame_name() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
