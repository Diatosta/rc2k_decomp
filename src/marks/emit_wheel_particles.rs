//! `marks::emit_wheel_particles` (0x4CA085): one wheel's particles. With the chance
//! the wheel's +0x34 by `flt_7651DC` (rounded, into `dword_9356C0`) out of 0x10000 (a
//! random's low word, `dword_935522`, under it) the wheel's +0x28 .. +0x30 (the kind and
//! two flags of the surface under it) become its +0x1C .. +0x24; then by the kind (+0x1C):
//! 1 `marks::emit_kind_1`; 2, 4, 6 .. 9, 0xC `marks::emit_bursts` and
//! `marks::emit_particle_6`; 3 `marks::emit_bursts_2` and `emit_particle_6`; 5
//! `marks::emit_particle_5`; 0xB `marks::emit_particle_11`; 0 and the rest nothing.
//!
//! In:  edi the wheel record (the car object's +0xD4, +0x11C, +0x164, +0x1AC), esi the car
//!      object (`dword_935416` the wheel's number, set by the caller)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Callers: `sub_4C9E2E`, four times. The original EXE the owner had patched turned this
//! into a `ret` (section 16, the burst loops' ring-full slip). Refuses a null wheel and the
//! emitters' refusals; on a refusal nothing stays written (all it may write is put back).

use crate::errors::{BadInput, Refused};
use crate::marks::emitters::*;
use crate::marks::store::{Slot, game_read, game_write};
use crate::marks::{
    emit_bursts, emit_bursts_2, emit_kind_1, emit_particle_5, emit_particle_6, emit_particle_11,
    emit_particle_18,
};
use crate::math::x87;

const CHANCE_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const CHANCE_K_RVA: Slot = Slot::at(0x7651DC); // flt_7651DC
const CHANCE_RANDOM_RVA: Slot = Slot::at(0x935522); // dword_935522

/// The chance and the random it would be met by, worked out without writing.
unsafe fn chance(wheel: u32) -> (i32, u32) {
    unsafe {
        let chance = x87::mul_to_int(at(wheel + 0x34), game_read(CHANCE_K_RVA));
        let r = game_read::<u32>(CHANCE_RANDOM_RVA)
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        (chance, r)
    }
}

/// Whether the copy is made.
fn copies(chance: i32, r: u32) -> bool {
    ((r & 0xFFFF) as i32) < chance
}

/// Whose switch: this one's, or `marks::emit_body_particles`' (its kind 1
/// `marks::emit_particle_18`, its kind 3 the bursts of kind 2: section 16).
#[derive(Clone, Copy, PartialEq)]
pub enum Table {
    Wheel,
    Body,
}

/// The emitters for a kind.
pub unsafe fn emit(table: Table, kind: u32, wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
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
}

/// The Rust version (see the header): `wheel` the wheel's record (the original's edi),
/// `object` the car's object (esi).
pub unsafe fn emit_wheel_particles(wheel: u32, object: u32) -> Result<(), Refused> {
    if wheel == 0 {
        return Err(BadInput("a null wheel").into());
    }
    unsafe {
        let (chance, _) = chance(wheel);
        game_write(CHANCE_RVA, chance);
        let r = next_c5(CHANCE_RANDOM_RVA);
        if copies(chance, r) {
            let surface = [at(wheel + 0x28), at(wheel + 0x2C), at(wheel + 0x30)];
            for (n, v) in surface.iter().enumerate() {
                ((wheel + 0x1C + 4 * n as u32) as *mut u32).write_unaligned(*v);
            }
        }
        emit(Table::Wheel, at(wheel + 0x1C), wheel, object)
    }
}
