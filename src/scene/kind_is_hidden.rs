//! `scene::kind_is_hidden` (0x452303): with `dword_71C3E0` not 0 or `dword_62389C` at 2,
//! whether the object eax is of kind 0x30 or 0x4E (`g_objectKinds`): CF set when it is,
//! clear otherwise (and clear without either setting).
//!
//! In:  eax the object
//! Out: CF; eax its kind when a setting was on, else kept; the rest kept (ZF as the last
//!      compare leaves it, not read: callers `jb`)
//!
//! Callers: `scene::object_hidden_by_kind` (0x4522F0, inlined in `scene::draw_shown_object`),
//! `scene::kind_hidden_at` (0x4522FC). Refuses an object past 0xFF while a setting is on (the
//! original reads past the table).

use crate::errors::Refused;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;

/// The Rust version: the object's (hidden, eax left).
pub fn kind_is_hidden(object: u32) -> Result<(bool, u32), Refused> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden`.
    let settings = rc2k_core::scene::kinds::HideSettings {
        switch: RACE_WORK.in_credits.bits(),
        mode: RACE_VARS.run_kind.bits(),
    };
    Ok(rc2k_core::scene::kinds::kind_is_hidden(
        settings,
        object,
        |n| {
            crate::scene::owner::scene()
                .objects
                .tables
                .object_kind(n as u32)
        },
    )?)
}
