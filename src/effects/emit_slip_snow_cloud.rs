//! Emits: the long-lived powder cloud a slipping wheel raises on snow.
//!
//! `effects::emit_slip_snow_cloud`: one ring entry of kind 0xB; `effects::emit_slip_spray`'s
//! code on its own variables (`SlipSprayLongSet.place_random`, `SlipSprayLongSet.motion_x`
//! .. `SlipSprayLongSet.spin`), a life of 2000, and made only when the wheel's |+0x18|
//! rounded to an integer is over 1 (kind 5: when it is not 0).
//!
//! Refuses as `emit_slip_spray`.

use crate::effects::emit_slip_spray::{JitterSet, emit_slip_spray_with};
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::memory::session::{PlainDword, PlainFloat};
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

/// Kind 0xB's emitter variables (`eleventh`).
#[derive(rc2k_derive::Cells)]
pub struct SlipSprayLongSet {
    /// The place's random
    pub place_random: PlainDword,
    /// The motion's x factor
    pub motion_x: PlainFloat,
    /// The motion's y factor
    pub motion_y: PlainFloat,
    /// Added to the motion's y
    pub lift: PlainFloat,
    /// The motion's z factor
    pub motion_z: PlainFloat,
    /// The spread `jitter_point` takes
    pub spread: PlainFloat,
    /// The third row's factors (z, y, x)
    pub roll_k: [PlainFloat; 3],
    /// The spin's factor
    pub spin: PlainFloat,
}

impl SlipSprayLongSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SlipSprayLongSet {
            place_random: PlainDword::new(0x0),
            motion_x: PlainFloat::new(10.0),
            motion_y: PlainFloat::new(10.0),
            lift: PlainFloat::new(15.0),
            motion_z: PlainFloat::new(10.0),
            spread: PlainFloat::new(110.0),
            roll_k: [
                PlainFloat::new(-12.0),
                PlainFloat::new(-12.0),
                PlainFloat::new(-12.0),
            ],
            spin: PlainFloat::new(-64.0),
        }
    }
}
