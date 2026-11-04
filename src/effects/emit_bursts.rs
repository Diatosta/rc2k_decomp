//! `effects::emit_bursts` (0x4CD21A): two bursts of ring entries from a wheel (kinds 0x11
//! and 0x10), when the wheel's second flags (+0x24, kept in `dword_939BAA`) >> 6 are not
//! 0. Each half: a strength (`flt_935694`: the wheel's +0x10 times a factor; the second
//! `flt_935698`: a blend of +0xC and +0x18) and its rate; when the half's random's bits
//! 16 .. 23 are under the rate, a direction (`flt_935688` ..) and a count (a random's low
//! 12 bits modulo (flags >> 6) times 4); then that many entries: their motion the
//! direction spread (`effects::spread_point`), a size, a look (+0x58), a turn from the car's
//! +0x4C .. (+0x38 ..), a place at the wheel less the car's matrix's place plus a random
//! side, a fade (+0x5C), a life of 2000, +0x7C / +0x80 `dword_935476` / `dword_93547A`.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! Callers: `effects::emit_wheel_particles`' kinds 2 .. 4, 6 .. 9, 0xC. The original's loops
//! push esi; when the ring is full (0x3FE taken) inside one they leave by 0x4CD741, which
//! takes only that push off: `popa` restores shifted registers and `retn` returns to a
//! register's value, the crash over mud (section 16; seen by the owner 2026-10-23). Fixed
//! here (2026-10-23): the bursts end at a full ring, as the exit with `lea esp, [esp+8]`
//! does. Refuses (before anything) a null wheel, a null object, car or matrix where read.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::field;
use crate::effects::emitters::*;
use crate::effects::surface_store::SURFACE;
use crate::effects::tuning::{MARKS_FLOATS, MARKS_FRAME};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::marks_settings_store::MarkCar;
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Dword, Float, read_bits, write_bits};

const LIFE: u32 = 0x7D0;

/// One half's variables.
pub struct Half {
    pub kind: u32,
    pub strength: &'static Float,
    /// The direction's factors (x, y, z) and whether |s| is scaled by `flt_765158` in y.
    pub factors: [&'static Float; 3],
    pub scaled: bool,
    /// The size's mask and base, the side's factors.
    pub size: (u32, u32),
    pub sides: [&'static Float; 3],
}

/// One function's variables: its two halves and the typed factors and randoms.
pub struct BurstSet {
    pub halves: [Half; 2],
    pub tuning: &'static BurstTuning,
}

/// The two halves' variables (`emit_bursts_2`'s are the same but for its tuning).
pub const HALVES: [Half; 2] = [
    Half {
        kind: 0x11,
        strength: &RING_WORK.strength[0],
        factors: [&EMITTER.side_a[0], &EMITTER.side_a[1], &EMITTER.side_a[2]],
        scaled: false,
        size: (0x7FF, 0x400),
        sides: [&EMITTER.side_b[0], &EMITTER.side_b[1], &EMITTER.side_b[2]],
    },
    Half {
        kind: 0x10,
        strength: &RING_WORK.strength[1],
        factors: [&EMITTER.side_b[0], &EMITTER.side_b[1], &EMITTER.side_b[2]],
        scaled: true,
        size: (0x3FF, 0x600),
        sides: [&EMITTER.side_a[0], &EMITTER.side_a[1], &EMITTER.side_a[2]],
    },
];

/// `emit_bursts`'.
pub static FIRST: BurstSet = BurstSet {
    halves: HALVES,
    tuning: &FIRST_TUNING,
};

impl BurstTuning {
    /// The half's (`n` 0 or 1) rate, count and entry randoms.
    pub fn randoms(&self, n: usize) -> &[Dword; 3] {
        if n == 0 { &self.first } else { &self.second }
    }
}

/// The half's strength and rate (`n` 0 or 1), from the wheel.
fn strength(set: &BurstSet, n: usize, wheel: GamePtr) -> Result<(u32, i32), Refused> {
    Ok({
        let scale: u32 = MARKS_FLOATS.thirty_two.bits();
        if n == 0 {
            x87::times_kept_abs_to_int(field(wheel, 0x10)?, set.tuning.strength_k.bits(), scale)
        } else {
            x87::blend_kept_abs_to_int(
                field(wheel, 0xC)?,
                field(wheel, 0x18)?,
                set.tuning.blend_k[0].bits(),
                set.tuning.blend_k[1].bits(),
                scale,
            )
        }
    })
}

/// The entries each half would make, worked out without writing.
fn counts(set: &BurstSet, wheel: GamePtr) -> Result<[u32; 2], Refused> {
    Ok({
        let flags2 = field(wheel, 0x24)?;
        if flags2 >> 6 == 0 {
            return Ok([0; 2]);
        }
        crate::effects::try_from_fn(|n| {
            let (_, rate) = strength(set, n, wheel)?;
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

/// Why the Rust version refuses, if it does, worked out without writing.
pub fn refusal(set: &BurstSet, wheel: GamePtr) -> Result<Option<&'static str>, Refused> {
    Ok({
        if field(wheel, 0x24)? >> 6 == 0 {
            return Ok(None);
        }
        let [a, b] = counts(set, wheel)?;
        if a + b > 0 && !car_with_matrix()? {
            return Ok(Some("a null object, car or matrix"));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_bursts(wheel: GamePtr, object: &'static MarkCar) -> Result<(), Refused> {
    emit_bursts_with(&FIRST, wheel, object)
}

/// The shared body, on `set`'s variables.
pub fn emit_bursts_with(
    set: &BurstSet,
    wheel: GamePtr,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(set, wheel)? {
            return Err(BadInput(why).into());
        }
        let flags2 = field(wheel, 0x24)?;
        SURFACE.wheel_flags2.set(flags2);
        if flags2 >> 6 == 0 {
            return Ok(());
        }
        for (n, h) in set.halves.iter().enumerate() {
            let (s, rate) = strength(set, n, wheel)?;
            h.strength.set_bits(s);
            RING_WORK.whole.set(rate as u32);
            let r = next_c5(&set.tuning.randoms(n)[0]);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                continue;
            }
            let car = emitting_car()?;
            let k: u32 = MARKS_FLOATS.minus_65_536.bits();
            let s: u32 = h.strength.bits();
            let scale = h.scaled.then(|| MARKS_FLOATS.half.bits());
            let direction = [
                x87::mul2(h.factors[0].bits(), s, k),
                x87::times_less_abs_times(h.factors[1].bits(), s, scale, k),
                x87::mul2(h.factors[2].bits(), s, k),
            ];
            write_bits(&RING_WORK.direction, direction);
            let count = (next_c5(&set.tuning.randoms(n)[1]) & 0xFFF) % ((flags2 >> 6) << 2);
            for _ in 0..count {
                // The ring full (0x3FE taken): the original leaves by 0x4CD741 / 0x4CDC6E,
                // whose stack slip crashed the game (section 16). Fixed 2026-10-23: the
                // bursts end there, the caller's registers kept, as `lea esp, [esp+8]`
                // would leave them.
                if RING_WORK.count.get() == MOST_TAKEN {
                    return Ok(());
                }
                let e = take_entry(object.word(0x1C)? & 0xFF_FFFF, h.kind)?;
                let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
                let d = read_bits(&RING_WORK.direction);
                let (out, _) = crate::effects::spread_point::spread_point([
                    extended(MARKS_FLOATS.half.bits()),
                    extended(d[2]),
                    extended(d[1]),
                    extended(d[0]),
                ]);
                put(0x34, rounded(out[0]))?;
                put(0x30, rounded(out[1]))?;
                put(0x2C, rounded(out[2]))?;
                let r = next_c5(&set.tuning.randoms(n)[2]);
                let size = (r.rotate_right(8) & h.size.0) + h.size.1;
                MARKS_FRAME.emit_side.set_bits(size);
                put(0x60, x87::from_int(size as i32))?;
                put(0x58, ((flags2 & 0x3F) << 4) + (r.rotate_right(17) & 3))?;
                let tk: u32 = MARKS_FLOATS.minus_eighth.bits();
                put(0x40, x87::mul(car.word(0x54)?, tk))?;
                put(0x3C, x87::mul(car.word(0x50)?, tk))?;
                put(0x38, x87::mul(car.word(0x4C)?, tk))?;
                for o in [0x44, 0x48, 0x4C] {
                    put(o, 0)?;
                }
                let m = car.whole()?;
                let side = (r.rotate_right(8) & 0x3FFF).wrapping_sub(0x2000);
                MARKS_FRAME.emit_side.set_bits(side);
                let place: [u32; 3] = crate::effects::try_from_fn(|k| {
                    let k4 = 4 * k as u32;
                    Ok(x87::less_plus_int_times(
                        field(wheel, 0x3C + k4)?,
                        matrix_cell(m, 0x30 + k4)?,
                        h.sides[k].bits(),
                        side as i32,
                    ))
                })?;
                put(0x10, place[0])?;
                put(0x14, place[1])?;
                put(0x28, place[1])?;
                put(0x18, place[2])?;
                let pick = r & 0x7_FFFF;
                RING_WORK.whole.set(pick);
                put(
                    0x5C,
                    x87::int_times_plus(
                        pick as i32,
                        MARKS_FLOATS.over_65536.bits(),
                        MARKS_FLOATS.one.bits(),
                    ),
                )?;
                put(0x84, LIFE)?;
                put(0x7C, EMITTER.looks[0].get())?;
                put(0x80, EMITTER.looks[1].get())?;
                put(0x68, EMITTER.car.get())?;
            }
        }
    }
    Ok(())
}

/// One burst emitter's factors and randoms (`emit_bursts`' at 0x93A3D2, `emit_bursts_2`'s at 0x93A3F6) (IDA 0x93A3D2 .. 0x93A3F6, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct BurstTuning {
    /// 0x93A3D2: the strength's factor
    pub strength_k: Float,
    /// 0x93A3D6: the first half's rate, count and entry randoms
    pub first: [Dword; 3],
    /// 0x93A3E2: the blend's two factors
    pub blend_k: [Float; 2],
    /// 0x93A3EA: the second half's rate, count and entry randoms
    pub second: [Dword; 3],
}

/// One burst emitter's factors and randoms (`emit_bursts`' at 0x93A3D2, `emit_bursts_2`'s at 0x93A3F6) (IDA 0x93A3D2 .. 0x93A3F6).
pub static FIRST_TUNING: BurstTuning = BurstTuning {
    strength_k: Float::from_bits(0xBE800000),
    first: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    blend_k: [Float::from_bits(0xBE800000), Float::from_bits(0xBD4CCCCD)],
    second: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
};

impl BurstTuning {
    pub const LEN: usize = 0x24;
}

/// Marker for `moved::MOVED`: FIRST_TUNING's place in the image.
pub struct FirstTuning;

impl FirstTuning {
    pub const START: usize = 0x93A3D2;
    pub const LEN: usize = BurstTuning::LEN;

    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const FIRST_TUNING) as usize
    }
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
            bytes(&FIRST_TUNING) == crate::moved::image_bytes(FirstTuning::START, BurstTuning::LEN),
            "FIRST_TUNING starts differently from the image"
        );
        assert_eq!(
            offset_of!(BurstTuning, strength_k),
            0x93A3D2 - FirstTuning::START
        );
        assert_eq!(
            offset_of!(BurstTuning, first),
            0x93A3D6 - FirstTuning::START
        );
        assert_eq!(
            offset_of!(BurstTuning, blend_k),
            0x93A3E2 - FirstTuning::START
        );
        assert_eq!(
            offset_of!(BurstTuning, second),
            0x93A3EA - FirstTuning::START
        );
    }
}
