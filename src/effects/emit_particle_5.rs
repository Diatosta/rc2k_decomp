//! `effects::emit_particle_5` (0x4CC15D, a chunk of `effects::emit_wheel_particles` entered by
//! `jz` from it and from 0x4CA20B, left by `popa; retn`): one ring entry of kind 5 from a
//! wheel whose +0x18 is not 0.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The strength `flt_93555E`: |the wheel's +0x18|, then less (bit 0 of `dword_935416`
//! clear, `dword_935432` -1) or plus (set, 1) its +0x10. The entry: its place the wheel's
//! (+0x3C ..) less the car matrix's (+0x30 ..) plus a random side (`dword_93554A` /
//! `dword_93554E`: the matrix's first and third rows by a random's low 14 bits, the first
//! less 0x2000); its motion the matrix's first row by `flt_93A23E` .. and the sign, by the
//! strength (the y plus `flt_93A246`), moved by `effects::jitter_point` (spread
//! `flt_93A24E`), plus the third row by `flt_93A252` .. and the wheel's +0x18; +0x54
//! `flt_93A25E` times the sign; +0x58 the wheel's flags' low 6 bits; +0x60 the integer
//! ((`jitter_point`'s ebx >> 2) & 0xFFF) + 0x7800 (section 16); +0x74 0xFF000000; a life of
//! 5000; +0x68 the car. Refuses (before anything) a null wheel, object, car or matrix where
//! read. The body is shared with kind 0xB's (`JitterSet`, `emit_particle_5_with`,
//! `test_with`).

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::ring_entries::field;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::{MarkCar, MarkRecord};
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Dword, Float};

/// One emitter's variables (kind 5's and kind 0xB's code is the same but for these).
pub struct JitterSet {
    pub kind: u32,
    /// Whether the rounded |speed| must be over 1 (kind 0xB) or the speed not 0 (kind 5).
    pub over_one: bool,
    pub place_random: &'static Dword,
    /// The motion's factors (x, y, z); `lift` the y's.
    pub motion_k: [&'static Float; 3],
    pub lift: &'static Float,
    pub spread: &'static Float,
    /// The third row's factors (z, y, x).
    pub roll_k: [&'static Float; 3],
    pub spin: &'static Float,
    pub life: u32,
}

/// Kind 5's (`dword_93A23A`, `flt_93A23E` .. `flt_93A25E`).
pub static FIFTH: JitterSet = JitterSet {
    kind: 5,
    over_one: false,
    place_random: &crate::effects::owner::EFFECTS
        .particles
        .kind5_set
        .place_random,
    motion_k: [
        &crate::effects::owner::EFFECTS.particles.kind5_set.motion_x,
        &crate::effects::owner::EFFECTS.particles.kind5_set.motion_y,
        &crate::effects::owner::EFFECTS.particles.kind5_set.motion_z,
    ],
    lift: &crate::effects::owner::EFFECTS.particles.kind5_set.lift,
    spread: &crate::effects::owner::EFFECTS.particles.kind5_set.spread,
    roll_k: [
        &crate::effects::owner::EFFECTS.particles.kind5_set.roll_k[0],
        &crate::effects::owner::EFFECTS.particles.kind5_set.roll_k[1],
        &crate::effects::owner::EFFECTS.particles.kind5_set.roll_k[2],
    ],
    spin: &crate::effects::owner::EFFECTS.particles.kind5_set.spin,
    life: 0x1388,
};

/// Why the Rust version refuses these registers, if it does, worked out without writing.
pub fn refusal(set: &JitterSet, wheel: &MarkRecord) -> Result<Option<&'static str>, Refused> {
    Ok({
        if !emits(set, wheel)? {
            return Ok(None);
        }
        // The ring full: nothing more is read.
        next_entry()?;
        if !car_with_matrix()? {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

/// Whether the wheel's +0x18 passes the set's test.
fn emits(set: &JitterSet, wheel: &MarkRecord) -> Result<bool, Refused> {
    Ok({
        let speed = wheel.speed_18.bits() & 0x7FFF_FFFF;
        if set.over_one {
            x87::to_int(speed) > 1
        } else {
            x87::loaded(speed) != 0
        }
    })
}

/// The Rust version (see the header).
pub fn emit_particle_5(wheel: &MarkRecord, object: &'static MarkCar) -> Result<(), Refused> {
    emit_particle_5_with(&FIFTH, wheel, object)
}

/// The shared body, on `set`'s variables.
pub fn emit_particle_5_with(
    set: &JitterSet,
    wheel: &MarkRecord,
    object: &'static MarkCar,
) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(set, wheel)? {
            return Err(BadInput(why).into());
        }
        let strength = x87::loaded(wheel.speed_18.bits()) & 0x7FFF_FFFF;
        EMITTER.strength.set_bits(strength);
        if !emits(set, wheel)? || next_entry()?.is_none() {
            return Ok(());
        }
        let e = take_entry(object.word(0x1C)? & 0xFF_FFFF, set.kind)?;
        let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
        let (sign, strength) = if EMITTER.record.get() & 1 == 0 {
            (-1i32, x87::sub(strength, wheel.speed_10.bits()))
        } else {
            (1, x87::add(strength, wheel.speed_10.bits()))
        };
        EMITTER.sign.set(sign as u32);
        EMITTER.strength.set_bits(strength);
        let car = emitting_car()?;
        let m = car.whole()?;
        let r = next_c5(set.place_random);
        let (a, b) = (((r & 0x3FFF) as i32) - 0x2000, (r & 0x3FFF) as i32);
        EMITTER.sides[0].set(a as u32);
        EMITTER.sides[1].set(b as u32);
        let place: [u32; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k = 4 * k as u32;
                x87::pair_sum_less(
                    a,
                    matrix_cell(m, k)?,
                    b,
                    matrix_cell(m, 0x20 + k)?,
                    wheel.place[k as usize / 4].bits(),
                    matrix_cell(m, 0x30 + k)?,
                )
            })
        })?;
        put(field::PLACE_Z, place[2])?;
        put(field::GROUND_Y, place[1])?;
        put(field::PLACE_Y, place[1])?;
        put(field::PLACE_X, place[0])?;
        let motion: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let lift = (k == 1).then(|| set.lift.bits());
                x87::times_int_plus_times_80(
                    matrix_cell(m, 4 * k as u32)?,
                    set.motion_k[k].bits(),
                    sign,
                    lift,
                    strength,
                )
            })
        })?;
        let (moved, last) =
            jitter_point([extended(set.spread.bits()), motion[2], motion[1], motion[0]]);
        let speed = wheel.speed_18.bits();
        for (n, o) in [field::MOTION_Z, field::MOTION_Y, field::MOTION_X]
            .into_iter()
            .enumerate()
        {
            let row = matrix_cell(m, 0x28 - 4 * n as u32)?;
            put(
                o,
                x87::product_plus(row, set.roll_k[n].bits(), speed, rounded(moved[n])),
            )?;
        }
        put(
            field::ANGLE_RATE,
            x87::float_times_int_to_int(set.spin.bits(), sign) as u32,
        )?;
        put(field::CELL, wheel.flags.get() & 0x3F)?;
        put(field::SIZE, ((last >> 2) & 0xFFF) + 0x7800)?;
        put(field::ALPHA, 0xFF00_0000)?;
        put(field::LIFE, set.life)?;
        put(field::CAR, EMITTER.car.get())?;
    }
    Ok(())
}

/// Kind 5's emitter variables (`emit_particle_5::FIFTH`) (IDA 0x93A23A .. 0x93A262, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Kind5Set {
    /// 0x93A23A: `dword_93A23A`: the place's random
    pub place_random: Dword,
    /// 0x93A23E: `flt_93A23E`: the motion's x factor
    pub motion_x: Float,
    /// 0x93A242: `flt_93A242`: the motion's y factor
    pub motion_y: Float,
    /// 0x93A246: `flt_93A246`: added to the motion's y
    pub lift: Float,
    /// 0x93A24A: `flt_93A24A`: the motion's z factor
    pub motion_z: Float,
    /// 0x93A24E: `flt_93A24E`: the spread `jitter_point` takes
    pub spread: Float,
    /// 0x93A252: `flt_93A252` .. `flt_93A25A`: the third row's factors (z, y, x)
    pub roll_k: [Float; 3],
    /// 0x93A25E: `flt_93A25E`: the spin's factor
    pub spin: Float,
}

impl Kind5Set {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        Kind5Set {
            place_random: Dword::new(0x0),
            motion_x: Float::from_bits(0x41200000),
            motion_y: Float::from_bits(0x41200000),
            lift: Float::from_bits(0x41700000),
            motion_z: Float::from_bits(0x41200000),
            spread: Float::from_bits(0x42DC0000),
            roll_k: [
                Float::from_bits(0xC1400000),
                Float::from_bits(0xC1400000),
                Float::from_bits(0xC1400000),
            ],
            spin: Float::from_bits(0xC2800000),
        }
    }
}

impl Kind5Set {
    pub const START: usize = 0x93A23A;
    pub const LEN: usize = 0x28;
}

#[cfg(test)]
mod kind5_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind5_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind5Set>(), Kind5Set::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.particles.kind5_set)
                == crate::moved::image_bytes(Kind5Set::START, Kind5Set::LEN),
            "KIND5_SET starts differently from the image"
        );
        assert_eq!(
            offset_of!(Kind5Set, place_random),
            0x93A23A - Kind5Set::START
        );
        assert_eq!(offset_of!(Kind5Set, motion_x), 0x93A23E - Kind5Set::START);
        assert_eq!(offset_of!(Kind5Set, motion_y), 0x93A242 - Kind5Set::START);
        assert_eq!(offset_of!(Kind5Set, lift), 0x93A246 - Kind5Set::START);
        assert_eq!(offset_of!(Kind5Set, motion_z), 0x93A24A - Kind5Set::START);
        assert_eq!(offset_of!(Kind5Set, spread), 0x93A24E - Kind5Set::START);
        assert_eq!(offset_of!(Kind5Set, roll_k), 0x93A252 - Kind5Set::START);
        assert_eq!(offset_of!(Kind5Set, spin), 0x93A25E - Kind5Set::START);
    }
}
