//! `effects::emit_body_contact_particles` (0x4CDF3A, a chunk of `effects::emit_body_particles` entered
//! for the record's kind 1): ring entries of kind 0x12.
//!
//! Takes: the record (the car object's +0x1F4), the car object
//!
//! The wheel flags get the record's +0x20; the count (`whole`): the record's +0x38 by a
//! constant (held to 1 unless under another constant) times four times the flags >> 6,
//! rounded; that many entries (none for 0 or less, but 0x80000000 of them for 0x80000000),
//! until the ring is full. Each: its motion the side vector moved by `effects::jitter_point`
//! (a spread constant), y made positive; its place the record's (+0x3C ..), less a lift in y
//! at +0x28; +0x1C .. +0x24 0; a life of 300 (+0x78 a constant over it); +0x68 the emitting
//! car. The first entry's tag (+0x90) is the object's +0x1C, the later ones the previous
//! entry's (0; section 16). Refuses (before anything) a null record, and a null object when an
//! entry is made.

use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::scene::matrix3::{extended, rounded};

const LIFE: u32 = 0x12C;
const ONE: u32 = 0x3F80_0000; // fld1

/// The count (and the flags' part it is scaled by), worked out without writing.
fn count(
    effects: &crate::effects::owner::Effects,
    record: &MarkRecord,
) -> Result<(i32, i32), Refused> {
    let setup = &effects.setup;
    Ok({
        let n = ((record.flags.get() >> 6) << 2) as i32;
        let (size, k) = (record.size.bits(), setup.floats.thirty_second.bits());
        let under = x87::times_compare(size, k, &setup.floats.one) & 0x700 == 0x100;
        let count = if under {
            bh::to_int(bh::ld(size) * bh::ld(k) * bh::int(n))
        } else {
            x87::float_times_int_to_int(ONE, n)
        };
        (count, n)
    })
}

pub fn emit_body_contact_particles(
    masks: &Masks,
    ring: &Ring,
    record: &MarkRecord,
    object: &'static CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let flags = record.flags.get();
        masks.surface.wheel_flags.set(flags);
        let (count, n) = count(effects, record)?;
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
                None => object.tag.get(),
                // (The original leaves a register here holding the entry before's last x,
                // which is 0.)
                Some(b) => crate::effects::ring_entries::entry(ring, b)?.last_place[0],
            };
            let e = take_entry(ring, effects, tag & 0xFF_FFFF, 0x12)?;
            before = Some(e);
            let slot = crate::effects::ring_entries::Slot::new(ring, e);
            let m = effects
                .kinds
                .emitter
                .motion
                .each_ref()
                .map(|r| extended(r.get()));
            let (moved, _) = jitter_point(
                [
                    extended(effects.setup.floats.hundred_twenty_eight.get()),
                    m[2],
                    m[1],
                    m[0],
                ],
                effects,
            );
            slot.put(|en| &mut en.motion[2], rounded(moved[0]))?;
            slot.put(|en| &mut en.motion[1], rounded(moved[1]) & 0x7FFF_FFFF)?;
            slot.put(|en| &mut en.motion[0], rounded(moved[2]))?;
            slot.put(
                |en| &mut en.place[0],
                x87::loaded(&record.place[0]).to_bits(),
            )?;
            slot.put(
                |en| &mut en.place[1],
                x87::loaded(&record.place[1]).to_bits(),
            )?;
            slot.put(
                |en| &mut en.ground_y,
                x87::sub(&record.place[1], &effects.setup.floats.fixed_16_scale).to_bits(),
            )?;
            slot.put(
                |en| &mut en.place[2],
                x87::loaded(&record.place[2]).to_bits(),
            )?;
            for n in 0..3 {
                slot.put(|en| &mut en.last_place[n], 0)?;
            }
            slot.put(|en| &mut en.life, LIFE)?;
            slot.put(|en| &mut en.car, effects.kinds.emitter.car.get())?;
            slot.put(
                |en| &mut en.alpha_fade,
                x87::over_int(&effects.setup.floats.two_fifty_seven, LIFE as i32).to_bits(),
            )?;
        }
    }
    Ok(())
}
