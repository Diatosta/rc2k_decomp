//! `effects::emit_body_contact_particles`: ring entries of kind 0x12, for the record's kind
//! 1 (entered from `effects::emit_body_particles`).
//!
//! Takes: the record (the car object's body record, the last of its `records`), the car object.
//!
//! The wheel flags get the record's `flags`; the count (`whole`): the record's `size` by a
//! constant (held to 1 unless under another constant) times four times the flags >> 6,
//! rounded; that many entries (none for 0 or less, but 0x80000000 of them for 0x80000000),
//! until the ring is full. Each: its motion the side vector moved by `effects::jitter_point`
//! (a spread constant), y made positive; its place the record's `place`, its `ground_y` that y
//! less a lift; the last place 0; a life of 300 (`alpha_fade` a constant over it); `car` the
//! emitting car. The first entry's `car_tag` is the object's `tag`, the later ones the previous
//! entry's (0: a slip kept as is). Refuses (before anything) a null record, and a null
//! object when an entry is made.

use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::ring_store::Ring;
use crate::errors::Refused;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

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
        let (size, k) = (record.size.get(), setup.floats.thirty_second.get());
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
    ring: &Ring,
    record: &MarkRecord,
    object: &CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
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
                // The later tags are the entry before's last x, which is 0 (a slip kept as
                // is).
                Some(b) => crate::effects::ring_entries::entry(ring, b)?.last_place[0].to_bits(),
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
            slot.put(|en| &mut en.motion[2], rounded_value(moved[0]))?;
            slot.put(|en| &mut en.motion[1], rounded_value(moved[1]).abs())?;
            slot.put(|en| &mut en.motion[0], rounded_value(moved[2]))?;
            slot.put(|en| &mut en.place[0], x87::loaded(&record.place[0]))?;
            slot.put(|en| &mut en.place[1], x87::loaded(&record.place[1]))?;
            slot.put(
                |en| &mut en.ground_y,
                x87::sub(&record.place[1], &effects.setup.floats.fixed_16_scale),
            )?;
            slot.put(|en| &mut en.place[2], x87::loaded(&record.place[2]))?;
            for n in 0..3 {
                slot.put(|en| &mut en.last_place[n], 0.0)?;
            }
            slot.put(|en| &mut en.life, LIFE)?;
            slot.put(|en| &mut en.car, effects.kinds.emitter.car.get())?;
            slot.put(
                |en| &mut en.alpha_fade,
                x87::over_int(&effects.setup.floats.two_fifty_seven, LIFE as i32),
            )?;
        }
    }
    Ok(())
}
