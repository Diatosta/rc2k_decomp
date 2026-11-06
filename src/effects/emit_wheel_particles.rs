//! `effects::emit_wheel_particles` (0x4CA085): one wheel's particles. With the chance
//! the wheel's +0x34 by `flt_7651DC` (rounded, into `dword_9356C0`) out of 0x10000 (a
//! random's low word, `dword_935522`, under it) the wheel's +0x28 .. +0x30 (the kind and
//! two flags of the surface under it) become its +0x1C .. +0x24; then by the kind (+0x1C):
//! 1 `effects::emit_kind_1`; 2, 4, 6 .. 9, 0xC `effects::emit_bursts` and
//! `effects::emit_particle_6`; 3 `effects::emit_bursts_2` and `emit_particle_6`; 5
//! `effects::emit_particle_5`; 0xB `effects::emit_particle_11`; 0 and the rest nothing.
//!
//! In:  edi the wheel record (the car object's +0xD4, +0x11C, +0x164, +0x1AC), esi the car
//!      object (`dword_935416` the wheel's number, set by the caller)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Callers: `sub_4C9E2E`, four times. The original EXE the owner had patched turned this
//! into a `ret` (section 16, the burst loops' ring-full slip). Refuses a null wheel and the
//! emitters' refusals; on a refusal nothing stays written (all it may write is put back).

use crate::effects::emitters::*;
use crate::effects::ring_store::RING_WORK;
use crate::effects::tuning::MARKS_FLOATS;
use crate::effects::{
    emit_bursts, emit_bursts_2, emit_kind_1, emit_particle_5, emit_particle_6, emit_particle_11,
    emit_particle_18,
};
use crate::errors::Refused;
use crate::math::x87;
use crate::race::marks_settings_store::{MarkCar, MarkRecord};

/// The chance and the random it would be met by, worked out without writing.
fn chance(wheel: &MarkRecord) -> Result<(i32, u32), Refused> {
    Ok({
        let chance = x87::mul_to_int(wheel.chance.bits(), MARKS_FLOATS.two_pow_16.bits());
        let r = EMITTER
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
/// `effects::emit_particle_18`, its kind 3 the bursts of kind 2: section 16).
#[derive(Clone, Copy, PartialEq)]
pub enum Table {
    Wheel,
    Body,
}

/// The emitters for a kind.
pub fn emit(
    table: Table,
    kind: u32,
    wheel: &MarkRecord,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    match kind {
        1 if table == Table::Body => emit_particle_18::emit_particle_18(wheel, object),
        1 => emit_kind_1::emit_kind_1(wheel, object),
        3 if table == Table::Wheel => {
            emit_bursts_2::emit_bursts_2(wheel, object)?;
            emit_particle_6::emit_particle_6(wheel, object)
        }
        2..=4 | 6..=9 | 0xC => {
            emit_bursts::emit_bursts(wheel, object)?;
            emit_particle_6::emit_particle_6(wheel, object)
        }
        5 => emit_particle_5::emit_particle_5(wheel, object),
        0xB => emit_particle_11::emit_particle_11(wheel, object),
        _ => Ok(()),
    }
}

/// The Rust version (see the header): `wheel` the wheel's record (the original's edi),
/// `object` the car's object (esi).
pub fn emit_wheel_particles(wheel: &MarkRecord, object: &'static MarkCar) -> Result<(), Refused> {
    {
        let (chance, _) = chance(wheel)?;
        RING_WORK.whole.set(chance as u32);
        let r = next_c5(&EMITTER.wheel_random);
        if copies(chance, r) {
            wheel.take_surface();
        }
        emit(Table::Wheel, wheel.kind.get(), wheel, object)
    }
}
