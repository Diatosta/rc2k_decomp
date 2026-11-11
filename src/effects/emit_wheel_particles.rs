//! `effects::emit_wheel_particles` (0x4CA085): one wheel's particles. With the chance
//! the wheel's +0x34 by `flt_7651DC` (rounded, into `dword_9356C0`) out of 0x10000 (a
//! random's low word, `dword_935522`, under it) the wheel's +0x28 .. +0x30 (the kind and
//! two flags of the surface under it) become its +0x1C .. +0x24; then by the kind (+0x1C):
//! 1 `effects::emit_rolling_particles`; 2, 4, 6 .. 9, 0xC `effects::emit_bursts` and
//! `effects::emit_wheel_fade_particle`; 3 `effects::emit_bursts_kind_3` and `emit_wheel_fade_particle`; 5
//! `effects::emit_slip_spray`; 0xB `effects::emit_slip_spray_long`; 0 and the rest nothing.
//!
//! Callers: `effects::emit_car_particles`, four times. The original EXE the owner had patched
//! turned this into a `ret` (section 16, the burst loops' ring-full slip). Refuses a null wheel and
//! the emitters' refusals; on a refusal nothing stays written (all it may write is put back).

use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::effects::owner::effects;
use crate::effects::{
    emit_body_contact_particles, emit_bursts, emit_bursts_kind_3, emit_rolling_particles,
    emit_slip_spray, emit_slip_spray_long, emit_wheel_fade_particle,
};
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

/// The chance and the random it would be met by, worked out without writing.
fn chance(wheel: &MarkRecord) -> Result<(i32, u32), Refused> {
    let effects = effects();
    Ok({
        let chance = x87::mul_to_int(&wheel.chance, &effects.setup.floats.fixed_16_scale);
        let r = effects
            .kinds
            .emitter
            .wheel_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        (chance, r)
    })
}

/// Whether the copy is made.
fn copies(chance: i32, r: u32) -> bool {
    ((r & 0xFFFF) as i32) < chance
}

/// Whose switch: this one's, or `effects::emit_body_particles`' (its kind 1
/// `effects::emit_body_contact_particles`, its kind 3 the bursts of kind 2: section 16).
#[derive(Clone, Copy, PartialEq)]
pub enum Table {
    Wheel,
    Body,
}

/// The emitters for a kind.
pub fn emit(
    effects: &Effects,
    table: Table,
    kind: u32,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    let (masks, particles, ring) = (&effects.masks, &effects.particles, &effects.ring);
    match kind {
        1 if table == Table::Body => emit_body_contact_particles::emit_body_contact_particles(
            masks, ring, wheel, object, effects,
        ),
        1 => emit_rolling_particles::emit_rolling_particles(
            masks, particles, ring, wheel, object, effects,
        ),
        3 if table == Table::Wheel => {
            emit_bursts_kind_3::emit_bursts_kind_3(effects, wheel, object)?;
            emit_wheel_fade_particle::emit_wheel_fade_particle(
                masks, particles, ring, wheel, object, effects,
            )
        }
        2..=4 | 6..=9 | 0xC => {
            emit_bursts::emit_bursts(effects, wheel, object)?;
            emit_wheel_fade_particle::emit_wheel_fade_particle(
                masks, particles, ring, wheel, object, effects,
            )
        }
        5 => emit_slip_spray::emit_slip_spray(ring, wheel, object),
        0xB => emit_slip_spray_long::emit_slip_spray_long(ring, wheel, object),
        _ => Ok(()),
    }
}

/// `wheel` is the wheel's record,
/// `object` the car's object.
pub fn emit_wheel_particles(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    {
        let (chance, _) = chance(wheel)?;
        effects.ring.work.whole.set(chance as u32);
        let r = next_c5(&effects.kinds.emitter.wheel_random);
        if copies(chance, r) {
            wheel.take_surface();
        }
        emit(effects, Table::Wheel, wheel.kind.get(), wheel, object)
    }
}
