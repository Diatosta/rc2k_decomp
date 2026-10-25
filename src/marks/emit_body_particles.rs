//! `marks::emit_body_particles` (0x4CA13B): the particles of the car's fifth record
//! (+0x1F4, under the body). First the object's +0x90 times `flt_765174` (rounded), the
//! record's +4 plus 0x386A0, its +0 bit 0 flipped (`dword_935416` 2, or 3 when the bit is
//! now clear), its +0xC 0; then `marks::emit_wheel_particles`' chance (its own random
//! `dword_935526`) of copying the surface's kind and flags, and its switch, but for kind 1
//! (`marks::emit_particle_18`) and kind 3 (the bursts of kind 2: section 16).
//!
//! In:  edi the record, esi the car object
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C9E2E`'s end. Refuses a null record or object and the emitters' refusals;
//! on a refusal nothing stays written (all it may write is put back).

use crate::errors::{BadInput, Refused};
use crate::marks::emit_wheel_particles::{Table, emit};
use crate::marks::emitters::*;
use crate::marks::ring_store::RING_WORK;
use crate::marks::tuning::MARKS_FLOATS;
use crate::math::x87;
const PACE: u32 = 0x386A0;

/// The first part (see the header), written.
unsafe fn prepare(record: u32, object: u32) {
    unsafe {
        let size = x87::int_times_to_int(at(object + 0x90) as i32, MARKS_FLOATS.three.bits());
        ((object + 0x90) as *mut i32).write_unaligned(size);
        ((record + 4) as *mut u32).write_unaligned(at(record + 4).wrapping_add(PACE));
        EMITTER.record.set(2);
        let flip = at(record) ^ 1;
        (record as *mut u32).write_unaligned(flip);
        if flip & 1 == 0 {
            EMITTER.record.set(3);
        }
        ((record + 0xC) as *mut u32).write_unaligned(0);
        let chance = x87::mul_to_int(at(record + 0x34), MARKS_FLOATS.two_pow_16.bits());
        RING_WORK.whole.set(chance as u32);
        let r = next_c5(&EMITTER.body_random);
        if ((r & 0xFFFF) as i32) < chance {
            let surface = [at(record + 0x28), at(record + 0x2C), at(record + 0x30)];
            for (n, v) in surface.iter().enumerate() {
                ((record + 0x1C + 4 * n as u32) as *mut u32).write_unaligned(*v);
            }
        }
    }
}

/// The Rust version (see the header): `record` the body's record (the original's edi),
/// `object` the car's object (esi).
pub unsafe fn emit_body_particles(record: u32, object: u32) -> Result<(), Refused> {
    if record == 0 || object == 0 {
        return Err(BadInput("a null record or object").into());
    }
    unsafe {
        prepare(record, object);
        emit(Table::Body, at(record + 0x1C), record, object)
    }
}
