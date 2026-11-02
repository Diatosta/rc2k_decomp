//! `effects::emit_car_particles` (0x4C9E2E): a car's particles for the frame. First the
//! places of its four wheel records (+0x3C .. of the records at +0xD4, +0x11C, +0x164,
//! +0x1AC): from the object's whole numbers +0xC (z, by `flt_9354EE`; front +0x118, rear
//! +0x160), +0x10 (the other z, by `flt_9354F2`: +0x1A8, +0x1F0), +0x14 (x, by
//! `flt_9354F6` plus `flt_9354FA`: right +0x158, +0x1E8, negated left +0x110, +0x1A0) and
//! +0x18 (y, by `flt_9354FE` less `flt_935502` and `flt_935506`: all four), each turned
//! and placed by the car's matrix (`scene::rotate_point`); `dword_935476` |the first x|
//! and `dword_93547A` the two z's sizes by `flt_765158`, rounded. Then
//! `effects::emit_wheel_particles` for each wheel (`dword_935416` 0 .. 3; before the rear
//! ones the side factors `flt_93545E` .. the matrix's first row, `flt_93543A` .. its third
//! negated), the factors set from `flt_93550A` .., and `effects::emit_body_particles` for
//! the record at +0x1F4.
//!
//! In:  esi the car object
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: 0x4C9E0C. Refuses a null object or matrix and the emitters' refusals; on a
//! refusal nothing stays written (all it may write is put back).

use crate::effects::emit_body_particles::emit_body_particles;
use crate::effects::emit_wheel_particles::emit_wheel_particles;
use crate::effects::emitters::*;
use crate::effects::tuning::MARKS_FLOATS;
use crate::errors::{BadInput, Refused};
use crate::math::cells::At;
use crate::math::x87;
use crate::race::marks_settings_store::MarkCar;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::{read_bits, write_bits};

/// The wheel records, then the body's.
const RECORDS: [u32; 5] = [0xD4, 0x11C, 0x164, 0x1AC, 0x1F4];
/// The Rust version (see the header): `o` the car's object (the original's esi).
pub fn emit_car_particles(o: &'static MarkCar) -> Result<(), Refused> {
    if o.word(0x24)? == 0 {
        return Err(BadInput("a null car matrix").into());
    }
    let k = read_bits(&EMITTER.place_k);
    let z1 = x87::int_times(o.word(0xC)? as i32, k[0]);
    o.put(0x118, z1)?;
    o.put(0x160, z1)?;
    let z2 = x87::int_times(o.word(0x10)? as i32, k[1]);
    o.put(0x1A8, z2)?;
    o.put(0x1F0, z2)?;
    let (x, left) = x87::int_times_plus_and_negated(o.word(0x14)? as i32, k[2], k[3]);
    o.put(0x158, x)?;
    o.put(0x1E8, x)?;
    o.put(0x110, left)?;
    o.put(0x1A0, left)?;
    let y = x87::int_times_less_less(o.word(0x18)? as i32, k[4], k[5], k[6]);
    for a in [0x15C, 0x1EC, 0x114, 0x1A4] {
        o.put(a, y)?;
    }
    EMITTER.looks[0].set(x87::to_int(o.word(0x110)? & 0x7FFF_FFFF) as u32);
    EMITTER.looks[1].set(x87::abs_sum_times_to_int(
        o.word(0x118)?,
        o.word(0x1A8)?,
        MARKS_FLOATS.half.bits(),
    ) as u32);
    let m = o.word(0x24)?;
    for p in [0x110, 0x158, 0x1A0, 0x1E8] {
        let turned = rotate_point(
            &At(m),
            [
                extended(o.word(p)?),
                extended(o.word(p + 4)?),
                extended(o.word(p + 8)?),
            ],
        );
        for (n, v) in turned.iter().enumerate() {
            o.put(p + 4 * n as u32, rounded(*v))?;
        }
    }
    for (n, &r) in RECORDS[..4].iter().enumerate() {
        if n == 2 {
            write_bits(&EMITTER.side_a, [at(m)?, at(m + 4)?, at(m + 8)?]);
            write_bits(
                &EMITTER.side_b,
                crate::effects::try_from_fn(|i__| {
                    let f = [0x20, 0x24, 0x28][i__];
                    Ok(at(m + f)? ^ 0x8000_0000)
                })?,
            );
        }
        EMITTER.record.set(n as u32);
        emit_wheel_particles(o.at(r), o)?;
    }
    let sides: [u32; 6] = std::array::from_fn(|n| x87::loaded(EMITTER.body_k[n].bits()));
    write_bits(&EMITTER.side_a, [sides[0], sides[1], sides[2]]);
    write_bits(&EMITTER.side_b, [sides[3], sides[4], sides[5]]);
    emit_body_particles(o.at(RECORDS[4]), o)?;
    Ok(())
}
