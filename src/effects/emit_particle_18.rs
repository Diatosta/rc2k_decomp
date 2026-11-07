//! `effects::emit_particle_18` (0x4CDF3A, a chunk of `sub_4CA13B` entered by `jz` from
//! 0x4CA1C5 for the record's kind 1, left by `popa; retn`): ring entries of kind 0x12.
//!
//! Takes: the record (the car object's +0x1F4), the car object
//!
//! `g_marksWheelFlags` the record's +0x20; the count (`dword_9356C0`): the record's +0x38
//! by `flt_7651D0` (held to 1, `fld1`, unless under `flt_76516C`) times four times the
//! flags >> 6, rounded; that many entries (`dec; jns`: none for 0 or less, but for
//! 0x80000000), until the ring is full. Each: its motion `flt_93546A` .. moved by
//! `effects::jitter_point` (spread `flt_765198`), y made positive; its place the record's
//! (+0x3C ..), less `flt_7651DC` in y at +0x28; +0x1C .. +0x24 0; a life of 300 (+0x78
//! `flt_7651A4` over it); +0x68 `g_marksCar`. The loop reuses esi for the entry: only the
//! first entry's tag (+0x90) is the object's +0x1C, the later ones the previous entry's (0;
//! section 16). Refuses (before anything) a null record, and a null object when an entry is
//! made.

use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::mask_store::Masks;
use crate::effects::owner::effects;
use crate::effects::ring_entries::field;
use crate::effects::ring_store::Ring;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::marks_settings_store::{MarkCar, MarkRecord};
use crate::scene::matrix3::{extended, rounded};

const LIFE: u32 = 0x12C;
const ONE: u32 = 0x3F80_0000; // fld1

/// The count (and the flags' part it is scaled by), worked out without writing.
fn count(record: &MarkRecord) -> Result<(i32, i32), Refused> {
    let setup = &effects().setup;
    Ok({
        let n = ((record.flags.get() >> 6) << 2) as i32;
        let (size, k) = (record.size.bits(), setup.floats.over_32.bits());
        let under = x87::times_compare(size, k, setup.floats.one.bits()) & 0x700 == 0x100;
        let count = if under {
            x87::times_int_to_int(size, k, n)
        } else {
            x87::float_times_int_to_int(ONE, n)
        };
        (count, n)
    })
}

/// The Rust version (see the header).
pub fn emit_particle_18(
    masks: &Masks,
    ring: &Ring,
    record: &MarkRecord,
    object: &'static MarkCar,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let flags = record.flags.get();
        masks.surface.wheel_flags.set(flags);
        let (count, n) = count(record)?;
        ring.work.whole.set(n as u32);
        ring.work.whole.set(count as u32);
        let mut left = count;
        // The first entry's tag is the object's, each next one's the entry before's.
        let mut before: Option<u32> = None;
        loop {
            left = left.wrapping_sub(1);
            if left < 0 || next_entry(ring)?.is_none() {
                break;
            }
            let tag = match before {
                None => object.word(car_word::TAG)?,
                // (The original's esi: the entry before's last x, 0 here.)
                Some(b) => crate::effects::ring_entries::get(ring, b, field::LAST_PLACE_X)?,
            };
            let e = take_entry(ring, effects, tag & 0xFF_FFFF, 0x12)?;
            before = Some(e);
            let put = |o: u32, v: u32| crate::effects::ring_entries::put(ring, e, o, v);
            let m = effects
                .kinds
                .emitter
                .motion
                .each_ref()
                .map(|r| extended(r.bits()));
            let (moved, _) = jitter_point(
                [
                    extended(effects.setup.floats.hundred_twenty_eight.bits()),
                    m[2],
                    m[1],
                    m[0],
                ],
                effects,
            );
            put(field::MOTION_Z, rounded(moved[0]))?;
            put(field::MOTION_Y, rounded(moved[1]) & 0x7FFF_FFFF)?;
            put(field::MOTION_X, rounded(moved[2]))?;
            put(field::PLACE_X, x87::loaded(record.place[0].bits()))?;
            put(field::PLACE_Y, x87::loaded(record.place[1].bits()))?;
            put(
                field::GROUND_Y,
                x87::sub(
                    record.place[1].bits(),
                    effects.setup.floats.two_pow_16.bits(),
                ),
            )?;
            put(field::PLACE_Z, x87::loaded(record.place[2].bits()))?;
            for o in [
                field::LAST_PLACE_X,
                field::LAST_PLACE_Y,
                field::LAST_PLACE_Z,
            ] {
                put(o, 0)?;
            }
            put(field::LIFE, LIFE)?;
            put(field::CAR, effects.kinds.emitter.car.get())?;
            put(
                field::ALPHA_FADE,
                x87::over_int(effects.setup.floats.two_fifty_seven.bits(), LIFE as i32),
            )?;
        }
    }
    Ok(())
}
