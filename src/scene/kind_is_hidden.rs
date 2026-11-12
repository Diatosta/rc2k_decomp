//! `scene::kind_is_hidden` (0x452303): with `in_credits` not 0 or the run kind at 2,
//! whether the object is of kind 0x30 or 0x4E: CF set when it is,
//! clear otherwise (and clear without either setting).
//!
//! Takes: the object
//!
//! Callers: `scene::object_hidden_by_kind` (0x4522F0, inlined in `scene::draw_shown_object`),
//! `scene::kind_hidden_at` (0x4522FC). Refuses an object past 0xFF while a setting is on (the
//! original reads past the table).

use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

/// Whether the object is hidden.
pub fn kind_is_hidden(
    shared: &RaceShared,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(bool, u32), Refused> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden`.
    let settings = rc2k_core::scene::kinds::HideSettings {
        switch: shared.work.get().in_credits.is_set(),
        mode: shared.vars.get().run_kind.get(),
    };
    Ok(rc2k_core::scene::kinds::kind_is_hidden(
        settings,
        object,
        |n| scene.objects.tables.get().object_kind(n as u32),
    )?)
}
