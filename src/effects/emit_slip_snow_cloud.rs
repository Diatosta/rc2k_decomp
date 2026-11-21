//! Emits: the long-lived powder cloud a slipping wheel raises on snow.
//!
//! `effects::emit_slip_snow_cloud`: one ring entry of kind 0xB; `effects::emit_slip_spray`'s
//! code on its own variables (`SlipSpraySet.place_random`, `SlipSpraySet.motion_x`
//! .. `SlipSpraySet.spin` of kind 0xB's set), a life of 2000, and made only when the wheel's |`local_velocity_z`|
//! rounded to an integer is over 1 (kind 5: when it is not 0).
//!
//! Refuses as `emit_slip_spray`.

use crate::effects::emit_slip_spray::{JitterSet, emit_slip_spray_with};
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

pub(super) fn emit_slip_snow_cloud(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &CarEffects,
) -> Result<(), Refused> {
    let mut set = effects.particles.kind11_set.borrow_mut();
    emit_slip_spray_with(
        effects,
        &mut JitterSet::new(0xB, true, 0x7D0, &mut set),
        wheel,
        object,
    )
}
