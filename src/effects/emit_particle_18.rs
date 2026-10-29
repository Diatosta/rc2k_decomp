//! `effects::emit_particle_18` (0x4CDF3A, a chunk of `sub_4CA13B` entered by `jz` from
//! 0x4CA1C5 for the record's kind 1, left by `popa; retn`): ring entries of kind 0x12.
//!
//! In:  edi the record (the car object's +0x1F4), esi the car object
//! Out: every register kept
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
use crate::effects::surface_store::SURFACE;
use crate::effects::tuning::MARKS_FLOATS;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};

const LIFE: u32 = 0x12C;
const ONE: u32 = 0x3F80_0000; // fld1

/// The count (and the flags' part it is scaled by), worked out without writing.
fn count(record: u32) -> Result<(i32, i32), Refused> {
    Ok({
        let n = ((at(record + 0x20)? >> 6) << 2) as i32;
        let (size, k) = (at(record + 0x38)?, MARKS_FLOATS.over_32.bits());
        let under = x87::times_compare(size, k, MARKS_FLOATS.one.bits()) & 0x700 == 0x100;
        let count = if under {
            x87::times_int_to_int(size, k, n)
        } else {
            x87::float_times_int_to_int(ONE, n)
        };
        (count, n)
    })
}

/// How many entries are made, worked out without writing.
fn made(record: u32) -> Result<u32, Refused> {
    Ok({
        let (count, _) = count(record)?;
        let wanted = if count == i32::MIN {
            1 << 31
        } else {
            count.max(0) as u32
        };
        wanted.min(MOST_TAKEN.saturating_sub(RING_WORK.count.get()))
    })
}

/// The Rust version (see the header).
pub fn emit_particle_18(record: u32, object: u32) -> Result<(), Refused> {
    {
        if record == 0 {
            return Err(BadInput("a null record").into());
        }
        if object == 0 && made(record)? > 0 {
            return Err(BadInput("a null object").into());
        }
        let flags = at(record + 0x20)?;
        SURFACE.wheel_flags.set(flags);
        let (count, n) = count(record)?;
        RING_WORK.whole.set(n as u32);
        RING_WORK.whole.set(count as u32);
        let mut left = count;
        // The first entry's tag is the object's, each next one's the entry before's.
        let mut before: Option<u32> = None;
        loop {
            left = left.wrapping_sub(1);
            if left < 0 || next_entry()?.is_none() {
                break;
            }
            let tag = match before {
                None => at(object + 0x1C)?,
                Some(b) => crate::effects::ring_entries::get(b, 0x1C)?,
            };
            let e = take_entry(tag & 0xFF_FFFF, 0x12)?;
            before = Some(e);
            let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
            let m = EMITTER.motion.each_ref().map(|r| extended(r.bits()));
            let (moved, _) = jitter_point([
                extended(MARKS_FLOATS.hundred_twenty_eight.bits()),
                m[2],
                m[1],
                m[0],
            ]);
            put(0x34, rounded(moved[0]))?;
            put(0x30, rounded(moved[1]) & 0x7FFF_FFFF)?;
            put(0x2C, rounded(moved[2]))?;
            put(0x10, x87::loaded(at(record + 0x3C)?))?;
            put(0x14, x87::loaded(at(record + 0x40)?))?;
            put(
                0x28,
                x87::sub(at(record + 0x40)?, MARKS_FLOATS.two_pow_16.bits()),
            )?;
            put(0x18, x87::loaded(at(record + 0x44)?))?;
            for o in [0x1C, 0x20, 0x24] {
                put(o, 0)?;
            }
            put(0x84, LIFE)?;
            put(0x68, EMITTER.car.get())?;
            put(
                0x78,
                x87::over_int(MARKS_FLOATS.two_fifty_seven.bits(), LIFE as i32),
            )?;
        }
    }
    Ok(())
}
