//! `scene::kind_is_hidden`: with `in_credits` not 0 or the run kind at 2, whether the object is
//! of kind 0x30 or 0x4E (false without either setting). Callers: `scene::draw_shown_object` and
//! `scene::kind_hidden_at`. Refuses an object past
//! 0xFF while a setting is on.

use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

/// Whether the object is hidden.
pub fn kind_is_hidden(
    shared: &RaceShared,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(bool, u32), Refused> {
    // Logic: `rc2k_core::scene::kinds::kind_is_hidden`.
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
