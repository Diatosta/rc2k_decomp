//! `effects::emit_bursts` (0x4CD21A): two bursts of ring entries from a wheel (kinds 0x11
//! and 0x10), when the wheel's second flags (+0x24, kept as `wheel_flags2`) >> 6 are not 0.
//! Each half: a strength (the first the wheel's +0x10 times a factor; the second a blend of
//! +0xC and +0x18) and its rate; when the half's random's bits 16 .. 23 are under the rate, a
//! direction and a count (a random's low 12 bits modulo (flags >> 6) times 4); then that many
//! entries: their motion the direction spread (`effects::spread_point`), a size, a look
//! (+0x58), a turn from the car's +0x4C .. (+0x38 ..), a place at the wheel less the car
//! matrix's place plus a random side, a fade (+0x5C), a life of 2000, +0x7C / +0x80 the
//! `looks` values.
//!
//! Takes: the wheel record, the car object
//!
//! Callers: `effects::emit_wheel_particles`' kinds 2 .. 4, 6 .. 9, 0xC. In the original, when
//! the ring is full (0x3FE taken) inside a loop the exit leaves the stack shifted: `popa`
//! restores shifted registers and `retn` returns to a register's value, the crash over mud
//! (section 16). Here the bursts simply end at a full ring. Refuses (before anything) a null
//! wheel, a null object, car or matrix where read.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::scene::matrix3::{extended, rounded_value};
use crate::session::{PlainDword, PlainFloat, read_bits, write_bits};

const LIFE: u32 = 0x7D0;

/// One half's variables.
pub struct Half<'a> {
    pub kind: u32,
    pub strength: &'a PlainFloat,
    /// The direction's factors (x, y, z) and whether |s| is scaled by the half constant in y.
    pub factors: [&'a PlainFloat; 3],
    pub scaled: bool,
    /// The size's mask and base, the side's factors.
    pub size: (u32, u32),
    pub sides: [&'a PlainFloat; 3],
}

/// One function's variables: its two halves and the typed factors and randoms.
pub struct BurstSet<'a> {
    pub halves: [Half<'a>; 2],
    pub tuning: &'a BurstTuning,
}

/// The two halves' variables (`emit_bursts_kind_3`'s are the same but for its tuning).
pub fn halves(effects: &Effects) -> [Half<'_>; 2] {
    [
        Half {
            kind: 0x11,
            strength: &effects.ring.work.strength[0],
            factors: [
                &effects.kinds.emitter.side_a[0],
                &effects.kinds.emitter.side_a[1],
                &effects.kinds.emitter.side_a[2],
            ],
            scaled: false,
            size: (0x7FF, 0x400),
            sides: [
                &effects.kinds.emitter.side_b[0],
                &effects.kinds.emitter.side_b[1],
                &effects.kinds.emitter.side_b[2],
            ],
        },
        Half {
            kind: 0x10,
            strength: &effects.ring.work.strength[1],
            factors: [
                &effects.kinds.emitter.side_b[0],
                &effects.kinds.emitter.side_b[1],
                &effects.kinds.emitter.side_b[2],
            ],
            scaled: true,
            size: (0x3FF, 0x600),
            sides: [
                &effects.kinds.emitter.side_a[0],
                &effects.kinds.emitter.side_a[1],
                &effects.kinds.emitter.side_a[2],
            ],
        },
    ]
}

/// `emit_bursts`'.
pub fn first(effects: &Effects) -> BurstSet<'_> {
    BurstSet {
        halves: halves(effects),
        tuning: &effects.particles.first_tuning,
    }
}

impl BurstTuning {
    /// The half's (`n` 0 or 1) rate, count and entry randoms.
    pub fn randoms(&self, n: usize) -> &[PlainDword; 3] {
        if n == 0 { &self.first } else { &self.second }
    }
}

/// The half's strength and rate (`n` 0 or 1), from the wheel.
fn strength(
    effects: &Effects,
    set: &BurstSet<'_>,
    n: usize,
    wheel: &MarkRecord,
) -> Result<(f32, i32), Refused> {
    Ok({
        let scale = effects.setup.floats.thirty_two.get();
        if n == 0 {
            x87::times_kept_abs_to_int(&wheel.local_velocity_x, &set.tuning.strength_k, scale)
        } else {
            x87::blend_kept_abs_to_int(
                &wheel.driven_speed,
                &wheel.local_velocity_z,
                &set.tuning.blend_k[0],
                &set.tuning.blend_k[1],
                scale,
            )
        }
    })
}

/// The entries each half would make, worked out without writing.
fn counts(effects: &Effects, set: &BurstSet<'_>, wheel: &MarkRecord) -> Result<[u32; 2], Refused> {
    Ok({
        let flags2 = wheel.flags2.get();
        if flags2 >> 6 == 0 {
            return Ok([0; 2]);
        }
        crate::effects::try_from_fn(|n| {
            let (_, rate) = strength(effects, set, n, wheel)?;
            let r = set.tuning.randoms(n)[0]
                .get()
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                return Ok(0);
            }
            let c = set.tuning.randoms(n)[1]
                .get()
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            Ok((c & 0xFFF) % ((flags2 >> 6) << 2))
        })?
    })
}

/// Why the function refuses, if it does, worked out without writing.
pub fn refusal(
    effects: &Effects,
    set: &BurstSet<'_>,
    wheel: &MarkRecord,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if wheel.flags2.get() >> 6 == 0 {
            return Ok(None);
        }
        let [a, b] = counts(effects, set, wheel)?;
        if a + b > 0 && !car_with_matrix(effects)? {
            return Ok(Some("a null object, car or matrix"));
        }
        None
    })
}

pub fn emit_bursts(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    emit_bursts_with(effects, &first(effects), wheel, object)
}

/// The shared body, on `set`'s variables.
pub fn emit_bursts_with(
    effects: &Effects,
    set: &BurstSet<'_>,
    wheel: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    let (paths, masks, ring) = (&effects.paths, &effects.masks, &effects.ring);
    {
        if let Some(why) = refusal(effects, set, wheel)? {
            return Err(BadInput(why).into());
        }
        let flags2 = wheel.flags2.get();
        masks.surface.wheel_flags2.set(flags2);
        if flags2 >> 6 == 0 {
            return Ok(());
        }
        for (n, h) in set.halves.iter().enumerate() {
            let (s, rate) = strength(effects, set, n, wheel)?;
            h.strength.set(s);
            ring.work.whole.set(rate as u32);
            let r = next_c5(&set.tuning.randoms(n)[0]);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                continue;
            }
            let car = emitting_car(effects)?;
            let k = effects.setup.floats.burst_direction_k.get();
            let s = h.strength.get();
            let scale = h.scaled.then(|| effects.setup.floats.half.bits());
            let direction = [
                x87::mul2(h.factors[0], s, k),
                x87::times_less_abs_times(h.factors[1], s, scale, k),
                x87::mul2(h.factors[2], s, k),
            ];
            write_bits(&ring.work.direction, direction.map(f32::to_bits));
            let count = (next_c5(&set.tuning.randoms(n)[1]) & 0xFFF) % ((flags2 >> 6) << 2);
            for _ in 0..count {
                // The ring is full (0x3FE taken): the original's exit from here slips on its
                // stack and crashed the game (section 16). The bursts end here instead.
                if ring.work.count.get() == MOST_TAKEN {
                    return Ok(());
                }
                let e = take_entry(ring, effects, object.tag.get() & 0xFF_FFFF, h.kind)?;
                let slot = crate::effects::ring_entries::Slot::new(ring, e);
                let d = read_bits(&ring.work.direction);
                let (out, _) = crate::effects::spread_point::spread_point(
                    paths,
                    [
                        extended(effects.setup.floats.half.get()),
                        extended(d[2]),
                        extended(d[1]),
                        extended(d[0]),
                    ],
                    effects,
                );
                slot.put(|en| &mut en.motion[2], rounded_value(out[0]))?;
                slot.put(|en| &mut en.motion[1], rounded_value(out[1]))?;
                slot.put(|en| &mut en.motion[0], rounded_value(out[2]))?;
                let r = next_c5(&set.tuning.randoms(n)[2]);
                let size = (r.rotate_right(8) & h.size.0) + h.size.1;
                effects.setup.frame.emit_side.set_bits(size);
                slot.put(|en| &mut en.size, x87::from_int(size as i32).to_bits())?;
                slot.put(
                    |en| &mut en.cell,
                    ((flags2 & 0x3F) << 4) + (r.rotate_right(17) & 3),
                )?;
                let tk = effects.setup.floats.minus_eighth.get();
                slot.put(|en| &mut en.push[2], x87::mul(&car.motion[2], tk))?;
                slot.put(|en| &mut en.push[1], x87::mul(&car.motion[1], tk))?;
                slot.put(|en| &mut en.push[0], x87::mul(&car.motion[0], tk))?;
                for n in 0..3 {
                    slot.put(|en| &mut en.spare[n], 0)?;
                }
                let m = car.whole()?;
                let side = (r.rotate_right(8) & 0x3FFF).wrapping_sub(0x2000);
                effects.setup.frame.emit_side.set_bits(side);
                let place: [f32; 3] = crate::effects::try_from_fn(|k| {
                    let k4 = 4 * k as u32;
                    Ok(x87::less_plus_int_times(
                        &wheel.place[k4 as usize / 4],
                        matrix_cell(m, 0x30 + k4)?,
                        h.sides[k],
                        side as i32,
                    ))
                })?;
                slot.put(|en| &mut en.place[0], place[0])?;
                slot.put(|en| &mut en.place[1], place[1])?;
                slot.put(|en| &mut en.ground_y, place[1])?;
                slot.put(|en| &mut en.place[2], place[2])?;
                let pick = r & 0x7_FFFF;
                ring.work.whole.set(pick);
                slot.put(
                    |en| &mut en.cell_rate,
                    x87::int_times_plus(
                        pick as i32,
                        &effects.setup.floats.over_65536,
                        &effects.setup.floats.one,
                    )
                    .to_bits(),
                )?;
                slot.put(|en| &mut en.life, LIFE)?;
                slot.put(
                    |en| &mut en.box_limit[0],
                    effects.kinds.emitter.looks[0].get(),
                )?;
                slot.put(
                    |en| &mut en.box_limit[1],
                    effects.kinds.emitter.looks[1].get(),
                )?;
                slot.put(|en| &mut en.car, effects.kinds.emitter.car.get())?;
            }
        }
    }
    Ok(())
}

/// One burst emitter's factors and randoms (`emit_bursts`' at 0x93A3D2, `emit_bursts_kind_3`'s at 0x93A3F6).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct BurstTuning {
    /// 0x93A3D2: the strength's factor
    pub strength_k: PlainFloat,
    /// 0x93A3D6: the first half's rate, count and entry randoms
    pub first: [PlainDword; 3],
    /// 0x93A3E2: the blend's two factors
    pub blend_k: [PlainFloat; 2],
    /// 0x93A3EA: the second half's rate, count and entry randoms
    pub second: [PlainDword; 3],
}

impl BurstTuning {
    /// The values the game starts with (the image's bytes).
    pub const fn first() -> Self {
        BurstTuning {
            strength_k: PlainFloat::new(-0.25),
            first: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            blend_k: [PlainFloat::new(-0.25), PlainFloat::new(-0.05)],
            second: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
        }
    }
}

impl BurstTuning {
    pub const LEN: usize = 0x24;
}

#[cfg(test)]
mod first_tuning_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn first_tuning_starts_as_the_games() {
        assert_eq!(size_of::<BurstTuning>(), BurstTuning::LEN);
        assert!(
            bytes(&crate::effects::owner::effects().particles.first_tuning)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.particles.first_tuning/36"
                ),
            "FIRST_TUNING starts differently from the image"
        );
        assert_eq!(offset_of!(BurstTuning, strength_k), 0x0);
        assert_eq!(offset_of!(BurstTuning, first), 0x4);
        assert_eq!(offset_of!(BurstTuning, blend_k), 0x10);
        assert_eq!(offset_of!(BurstTuning, second), 0x18);
    }
}
