//! Emits: the long-lived powder cloud a slipping wheel raises on snow.
//!
//! `effects::emit_slip_snow_cloud`: one ring entry of kind 0xB; `effects::emit_slip_spray`'s
//! code on its own variables (`SlipSprayLongSet.place_random`, `SlipSprayLongSet.motion_x`
//! .. `SlipSprayLongSet.spin`), a life of 2000, and made only when the wheel's |`local_velocity_z`|
//! rounded to an integer is over 1 (kind 5: when it is not 0).
//!
//! Refuses as `emit_slip_spray`.

use crate::effects::emit_slip_spray::{JitterSet, emit_slip_spray_with};
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

/// Kind 0xB's variables.
pub fn eleventh(effects: &Effects) -> JitterSet<'_> {
    JitterSet {
        kind: 0xB,
        over_one: true,
        place_random: &effects.particles.kind11_set.place_random,
        motion_k: [
            &effects.particles.kind11_set.motion_x,
            &effects.particles.kind11_set.motion_y,
            &effects.particles.kind11_set.motion_z,
        ],
        lift: &effects.particles.kind11_set.lift,
        spread: &effects.particles.kind11_set.spread,
        roll_k: [
            &effects.particles.kind11_set.roll_k[0],
            &effects.particles.kind11_set.roll_k[1],
            &effects.particles.kind11_set.roll_k[2],
        ],
        spin: &effects.particles.kind11_set.spin,
        life: 0x7D0,
    }
}

pub(super) fn emit_slip_snow_cloud(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &CarEffects,
) -> Result<(), Refused> {
    emit_slip_spray_with(effects, &eleventh(effects), wheel, object)
}
