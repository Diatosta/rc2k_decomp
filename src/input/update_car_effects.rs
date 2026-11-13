//! `input::update_car_effects` (0x40F347): one car's force feedback for one device. From the
//! car's physics (the view block) and its sound object it fills the object's feedback inputs
//! and sends up to seven requests (`input::set_effect`) to the device:
//! - the object's `flag_ac` = the block's `front_wheels_off`; `effect_in` = the local velocity
//!   (x, then drive speed minus z, then x and z again) times ten, truncated (`fistp`);
//! - the knock: with a weight and an x component both not zero, (256 - |atan2(y, x)| *
//!   162.97) * weight * 0.1, else 0; the angle also goes into the session's scratch dword; the
//!   knock inputs are cleared afterwards;
//! - the surface effect: the surface's effect kind and the speed (|`sum_a`|, or at 5 or less
//!   the revs squared >> 19), held to 0x100;
//! - the steering and engine effects by the surface kind (5, 9 and 0; 7; 8; others), and for
//!   the others the car's landing (leaving the ground, coming back) or a steering force from
//!   `effect_in`;
//! - the knock effect once per knock (+-0x100; `knock_count` counts to 4 before the next);
//! - the speed-change effect: the change of `effect_in[3]` since the last time, held to +-0x200,
//!   halved.
//!
//! Every request is a "set" (what 3). The x87 work uses the original's rounding.
//! Refuses (before anything) a surface number past the table's 16.

use crate::errors::{BadInput, Refused};
use crate::input::store::ForceFeedback;
use crate::input::types::InputDevice;
use crate::math::x87::mul_to_int;
use crate::math::x87_float::{Loadable, ld, st, to_int};
use crate::system::owner::System;

/// The force feedback effects of a device's record (its six slots).
#[derive(Clone, Copy)]
enum Effect {
    Steering = 0,
    Surface = 1,
    Landing = 2,
    Knock = 3,
    /// Effect 4: set with the steering effect on every surface kind, to the same figure on kinds 5, 9, 0 (0x100) and 7 (0x55), else 0 (probably a second force on the same device; nothing else shows what it drives).
    Fourth = 4,
    SpeedChange = 5,
}

/// A "set" request of one effect with its two numbers; the second number is what the
/// next request that has none of its own repeats (the original passes it on in a register).
struct Requests<'a> {
    feedback: &'a ForceFeedback,
    device: &'a InputDevice,
    last_strength: u32,
}

impl Requests<'_> {
    fn set(&mut self, effect: Effect, a: u32, b: u32) -> Result<(), Refused> {
        self.last_strength = b;
        crate::input::set_effect::set_effect(self.feedback, self.device, effect as u32, 3, a, b)
    }
}

/// The number of surface kinds (the effect kind table's entries).
const KIND_COUNT: u32 = 16;

/// `fld a; fsub b; fmul c; fistp`.
fn sub_mul_to_int(a: impl Loadable, b: impl Loadable, c: impl Loadable) -> i32 {
    to_int((ld(a) - ld(b)) * ld(c))
}

/// `fld y; fld x; fpatan; fst angle; fabs; fmul s; fsubr full; fmul d; fmul tenth;
/// fistp`: (the angle's bits, the knock).
fn knock(
    y: u32,
    x: u32,
    s: impl Loadable,
    full: impl Loadable,
    d: u32,
    tenth: impl Loadable,
) -> (u32, i32) {
    let angle = ld(y).atan2(ld(x));
    let knock = (ld(full) - angle.abs() * ld(s)) * ld(d) * ld(tenth);
    (st(angle), to_int(knock))
}

/// The car's feedback for the frame, the angle written to `scratch`; the original's x87
/// arithmetic. `view` the view block, `o` the car's sound object, `device` the device's
/// record.
fn update_car_effects_with(
    effects: &ForceFeedback,
    view: &crate::physics::car_store::Car,
    o: &crate::sound::store::SoundObject,
    device: &InputDevice,
    scratch: &crate::session::PlainDword,
) -> Result<(), Refused> {
    let surface = o.surface.get();
    if surface >= KIND_COUNT {
        return Err(BadInput("a surface number past the effect kinds' 16").into());
    }
    let mut out = Requests {
        feedback: effects,
        device,
        last_strength: 0,
    };
    o.flag_ac.set(view.front_wheels_off.get());
    let ten = effects.effect_ten.get();
    o.effect_in[0].set(mul_to_int(&view.local_velocity[0], ten) as u32);
    o.effect_in[1]
        .set(sub_mul_to_int(view.drive_speed_a.get(), view.local_velocity[2].get(), ten) as u32);
    o.effect_in[2].set(mul_to_int(&view.local_velocity[0], ten) as u32);
    o.effect_in[3].set(mul_to_int(&view.local_velocity[2], ten) as u32);
    // The knock's force from its inputs (angle from y = `knock_in[2]`, x = `knock_in[0]`).
    if o.knock_in[3].bits() & 0x7FFF_FFFF != 0 && o.knock_in[0].bits() & 0x7FFF_FFFF != 0 {
        let (angle, force) = knock(
            o.knock_in[2].bits(),
            o.knock_in[0].bits(),
            effects.effect_angle_scale.get(),
            effects.effect_full.get(),
            o.knock_in[3].bits(),
            effects.effect_tenth.get(),
        );
        scratch.set(angle);
        o.knock.set(force as u32);
    } else {
        o.knock.set(0);
    }
    for d in &o.knock_in {
        d.set_bits(0);
    }
    // The surface effect: the surface's kind and the speed.
    let mut kind = effects.surface_effect_kinds[surface as usize].get();
    let mut speed = (o.sum_a.get() as i32).unsigned_abs();
    if speed <= 5 {
        kind = 0;
        let revs = o.revs.get();
        speed = revs.wrapping_mul(revs) >> 19;
    }
    let speed = speed.min(0x100);
    out.set(Effect::Surface, kind, speed)?;
    let strength = || o.effect_request_b.get();
    let mut landed_now = false;
    match kind {
        5 | 9 | 0 => {
            out.set(Effect::Fourth, 0x100, strength())?;
            out.set(Effect::Steering, 0, strength())?;
        }
        7 => {
            out.set(Effect::Fourth, 0x55, strength())?;
            out.set(Effect::Steering, 0x55, strength())?;
        }
        8 => {
            out.set(Effect::Fourth, 0, strength())?;
            out.set(Effect::Steering, 0, strength())?;
        }
        _ => {
            out.set(Effect::Fourth, 0, strength())?;
            if o.flag_ac.is_set() {
                // Wheels off the ground.
                o.landed.set(1);
                out.set(Effect::Steering, 0, strength())?;
                out.set(Effect::Surface, 0, 0x100)?;
                out.set(Effect::Fourth, 0, strength())?;
                landed_now = true;
            } else if o.landed.get() == 1 {
                // Back on the ground.
                o.landed.set(0);
                out.set(Effect::Steering, 0x100, strength())?;
                let carried = out.last_strength;
                out.set(Effect::Landing, 0x100, carried)?;
            } else {
                // A steering force from the sideways and forward velocities.
                let a = (o.effect_in[0].get() << 1).wrapping_add(o.effect_in[1].get()) << 1;
                let a = (a as i32).wrapping_abs();
                if a > 0x14 {
                    let a = ((a - 0x14) as u32).min(0x100);
                    out.set(Effect::Steering, 0x100 - a, strength())?;
                } else {
                    out.set(Effect::Steering, 0x100, strength())?;
                }
            }
        }
    }
    if !landed_now {
        // The knock effect, once per knock.
        let force = o.knock.get();
        let mut counted = false;
        if force != 0 && o.flag_ac.get() == 0 {
            if o.knock_count.get() == 0 {
                let push = if (force as i32) < 0 {
                    0xFFFF_FF00
                } else {
                    0x100
                };
                let carried = out.last_strength;
                out.set(Effect::Knock, push, carried)?;
                o.knock_count.set(o.knock_count.get().wrapping_add(1));
                counted = true;
            } else {
                o.knock.set(0);
            }
        }
        if !counted && o.knock_count.is_set() {
            o.knock_count.set(o.knock_count.get().wrapping_add(1));
            if o.knock_count.get() >= 4 {
                o.knock_count.set(0);
            }
        }
    }
    // The change of `effect_in[3]` since the last time.
    let now = o.effect_in[3].get();
    let before = o.last_forward_speed.get();
    o.last_forward_speed.set(now);
    let change = (now.wrapping_sub(before) as i32).clamp(-0x200, 0x200) >> 1;
    let carried = out.last_strength;
    out.set(Effect::SpeedChange, change as u32, carried)
}

/// The car's force feedback for the frame (the original's x87 arithmetic). The
/// converted version (`rc2k_core::input::feedback::update_car_effects`, plain floats)
/// waits for the float pass.
pub fn update_car_effects(
    effects: &ForceFeedback,
    view: &crate::physics::car_store::Car,
    object: &crate::sound::store::SoundObject,
    device: &InputDevice,
    system: &System,
) -> Result<(), Refused> {
    update_car_effects_with(effects, view, object, device, &system.machine().scratch)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn sub_mul_to_int_matches_the_assembly() {
        let test = Differential {
            name: "input.update_car_effects.sub_mul_to_int",
            cases: 6000,
            seed: 209,
            kinds: [Kind::Exact; 4],
            fill: &|rng, m| {
                for d in &mut m[..3] {
                    *d = if rng.below(2) == 0 {
                        rng.moderate()
                    } else {
                        rng.fl()
                    };
                }
                m[3] = 0xAAAA_AAAA;
            },
            plain: &|c| c[3].set(sub_mul_to_int(c[0].get(), c[1].get(), c[2].get()) as u32),
        };
        differential(test);
    }

    #[test]
    fn knock_matches_the_assembly() {
        let mut kinds = [Kind::Exact; 8];
        kinds[6] = Kind::Ulp1;
        let test = Differential {
            name: "input.update_car_effects.knock",
            cases: 6000,
            seed: 210,
            kinds,
            fill: &|rng, m| {
                for d in &mut m[..6] {
                    *d = rng.moderate();
                }
                m[6] = 0xAAAA_AAAA;
                m[7] = 0xAAAA_AAAA;
            },
            plain: &|c| {
                let v: Vec<u32> = c[..6].iter().map(std::cell::Cell::get).collect();
                let (angle, force) = knock(v[0], v[1], v[2], v[3], v[4], v[5]);
                c[6].set(angle);
                c[7].set(force as u32);
            },
        };
        differential(test);
    }
}
