//! `scene::kind_is_hidden` (0x452303): with `dword_71C3E0` not 0 or `dword_62389C` at 2,
//! whether the object eax is of kind 0x30 or 0x4E (`g_objectKinds`): CF set when it is,
//! clear otherwise (and clear without either setting).
//!
//! In:  eax the object
//! Out: CF; eax its kind when a setting was on, else kept; the rest kept (ZF as the last
//!      compare leaves it, not read: callers `jb`)
//!
//! Callers: `scene::object_hidden_by_kind` (0x4522F0, inlined in
//! `scene::draw_shown_object`), `scene::kind_hidden_at` (0x4522FC).
//! Refuses an object past 0xFF while a setting is on (the original reads past the table).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): both settings, objects 0 .. 0xFF with test kinds.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const SWITCH_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const MODE_RVA: usize = rva(0x62389C); // dword_62389C
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds (0x100 dwords)

/// The Rust version: the object's (hidden, eax left).
pub unsafe fn kind_is_hidden(object: u32) -> Result<(bool, u32), Refused> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden`.
    let settings = rc2k_core::scene::kinds::HideSettings {
        switch: unsafe { game_read(SWITCH_RVA) },
        mode: unsafe { game_read(MODE_RVA) },
    };
    Ok(rc2k_core::scene::kinds::kind_is_hidden(
        settings,
        object,
        |n| unsafe { game_read(KINDS_RVA + 4 * n) },
    )?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (hidden, eax) = unsafe { kind_is_hidden(r.eax) }?;
    Ok(Left::carry(hidden).with(EAX, eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
