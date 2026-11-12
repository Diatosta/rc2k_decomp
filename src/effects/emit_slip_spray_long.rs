//! `effects::emit_slip_spray_long` (0x4CC35A, a chunk of `effects::emit_wheel_particles` entered
//! by `jz` from it and from 0x4CA215, left by `popa; retn`): one ring entry of kind 0xB;
//! `effects::emit_slip_spray`'s code on its own variables (`SlipSprayLongSet.place_random`, `SlipSprayLongSet.motion_x` ..
//! `SlipSprayLongSet.spin`), a life of 2000, and made only when the wheel's |+0x18| rounded to an
//! integer (`fist`, the x87's rounding) is over 1 (kind 5: when it is not 0).
//!
//! Takes: the wheel record, the car object
//!
//! Refuses as `emit_slip_spray`.

use crate::effects::emit_slip_spray::{JitterSet, emit_slip_spray_with};
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::session::{PlainDword, PlainFloat};

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

pub fn emit_slip_spray_long(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_slip_spray_with(effects, &eleventh(effects), wheel, object)
}

/// Kind 0xB's emitter variables (`emit_slip_spray::ELEVENTH`), and the constants of code that is in no function (IDA 0x93A262 .. 0x93A312).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SlipSprayLongSet {
    /// 0x93A262: `dword_93A262`: the place's random
    pub place_random: PlainDword,
    /// 0x93A266: `flt_93A266`: the motion's x factor
    pub motion_x: PlainFloat,
    /// 0x93A26A: `flt_93A26A`: the motion's y factor
    pub motion_y: PlainFloat,
    /// 0x93A26E: `flt_93A26E`: added to the motion's y
    pub lift: PlainFloat,
    /// 0x93A272: `flt_93A272`: the motion's z factor
    pub motion_z: PlainFloat,
    /// 0x93A276: `flt_93A276`: the spread `jitter_point` takes
    pub spread: PlainFloat,
    /// 0x93A27A: `flt_93A27A` .. `flt_93A282`: the third row's factors (z, y, x)
    pub roll_k: [PlainFloat; 3],
    /// 0x93A286: `flt_93A286`: the spin's factor
    pub spin: PlainFloat,
}

impl SlipSprayLongSet {
    /// The values the game starts with (the image's bytes).
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
