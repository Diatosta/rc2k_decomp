//! `effects::emit_kind_1` (0x4CCCD7, a chunk of `effects::emit_wheel_particles` entered by
//! `jz` from it for the wheel kind 1): `effects::emit_particle_21` when `dword_93A1D6` is
//! not 0, else (after `pusha`) the body at 0x4CCF71, `emit_particle_1`: one ring entry of
//! kind 1.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! `emit_particle_1`: `g_marksWheelFlags` the wheel's +0x20; the wheel's +4 (a float here)
//! grows by |+0x18| + |+0x10| + |+0x18 - +0xC|; when that rounded (`fist`) is 0x18 or more
//! it is kept as its remainder by 0x18 (as a float) and an entry may be made: the strength
//! (`dword_9356C0`, a float: |+0x10| by `flt_765168` plus |+0xC - +0x18|, by `flt_7651D0`)
//! must not be under `flt_765164` (compared unrounded), and is held to at most 1
//! (`flt_76516C`); the life (`dword_93567C`) it by `flt_7651F0` plus `flt_7651E8`, rounded;
//! the fade (`flt_935678`) it by `dword_76519C`. The entry: its place the wheel's (+0x3C
//! ..) plus a random side times `flt_93543A` .., less `flt_7651DC` in y at +0x28; its
//! motion the car matrix's first row by +0x10, its third by +0x18 and `flt_93543A` .. by
//! +0xC, each by a factor (`flt_93A39E` ..), moved by `effects::jitter_point` (spread
//! `flt_93A3C2`), y plus `flt_93A3C6`; a random spin and turn; the flags' low 6 bits; the
//! life and fade (+0x78 fade over life, +0x74 fade); the car and its +0x88; +0x70 0x10000
//! and +0x60 it by `flt_93A3CE`; +0x64 0. Refuses (before anything) a null wheel, a null
//! object where read, a null car or matrix when an entry is made, and `emit_particle_21`'s.

use crate::effects::emit_particle_21::emit_particle_21;
use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::field;
use crate::effects::emitters::*;
use crate::effects::jitter_point::jitter_point;
use crate::effects::ring_store::RING_WORK;
use crate::effects::surface_store::SURFACE;
use crate::effects::tuning::{MARKS_FLOATS, MARKS_FRAME};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::marks_settings_store::MarkCar;
use crate::scene::matrix3::{extended, rounded};
use crate::session::{Dword, Float};

/// flt_765168, flt_7651D0, flt_765164: the strength's factors and its least.
static STRENGTH_K: [&Float; 3] = [
    &MARKS_FLOATS.three_tenths,
    &MARKS_FLOATS.over_32,
    &MARKS_FLOATS.fifth,
];
/// flt_7651F0, flt_7651E8: the life's factor and base.
static LIFE_K: [&Float; 2] = [&MARKS_FLOATS.two_thousand, &MARKS_FLOATS.hundred];
const SIZE: u32 = 0x1_0000;
const STEP: i32 = 0x18;

/// The first part of `emit_particle_1`, worked out without writing: the wheel's +4 grown
/// (as `fist` and as a float), and, when an entry may be made, the strength (the float
/// kept, before the hold to 1).
struct Pace {
    grown: (i32, u32),
    strength: Option<(u32, bool)>,
}

fn pace(wheel: GamePtr) -> Result<Pace, Refused> {
    Ok({
        let grown = x87::abs_sum_to_int_kept(
            field(wheel, 0x18)?,
            field(wheel, 0x10)?,
            field(wheel, 0xC)?,
            field(wheel, 4)?,
        );
        if grown.0 < STEP {
            return Ok(Pace {
                grown,
                strength: None,
            });
        }
        let [k1, k2, least] = STRENGTH_K.map(super::super::session::Float::bits);
        let (kept, status) = x87::abs_blend_compare(
            field(wheel, 0x10)?,
            k1,
            field(wheel, 0xC)?,
            field(wheel, 0x18)?,
            k2,
            least,
        );
        let under = status & 0x700 == 0x100;
        Pace {
            grown,
            strength: Some((kept, under)),
        }
    })
}

/// Why `emit_particle_1` refuses, if it does, worked out without writing.
fn refusal(wheel: GamePtr) -> Result<Option<&'static str>, Refused> {
    Ok({
        match pace(wheel)?.strength {
            Some((_, false)) => {}
            _ => return Ok(None),
        }
        next_entry()?;
        if !car_with_matrix()? {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

/// The body at 0x4CCF71 (see the header).
pub fn emit_particle_1(wheel: GamePtr, object: &'static MarkCar) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(wheel)? {
            return Err(BadInput(why).into());
        }
        let flags = field(wheel, 0x20)?;
        SURFACE.wheel_flags.set(flags);
        let p = pace(wheel)?;
        RING_WORK.whole.set(p.grown.0 as u32);
        wheel.offset(4).write::<u32>(p.grown.1, "a mark's field")?;
        let Some((kept, under)) = p.strength else {
            return Ok(());
        };
        let rest = p.grown.0 % STEP;
        RING_WORK.whole.set(rest as u32);
        wheel
            .offset(4)
            .write::<u32>(x87::from_int(rest), "a mark's field")?;
        RING_WORK.whole.set(kept);
        if under {
            return Ok(());
        }
        let one: u32 = MARKS_FLOATS.one.bits();
        if x87::compare_status(kept, one) != 0x100 {
            RING_WORK.whole.set(one);
        }
        let strength: u32 = RING_WORK.whole.get();
        let life = x87::times_plus_to_int(strength, LIFE_K[0].bits(), LIFE_K[1].bits());
        RING_WORK.life.set(life as u32);
        let fade = x87::mul(strength, MARKS_FLOATS.two_fifty_five.bits());
        RING_WORK.fade.set_bits(fade);
        if next_entry()?.is_none() {
            return Ok(());
        }
        let e = take_entry(object.word(0x1C)? & 0xFF_FFFF, 1)?;
        let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
        let car = emitting_car()?;
        let m = car.whole()?;
        let side = (next_c5(&KIND1_SET.side_random) & 0x3FFF) as i32 - 0x2000;
        MARKS_FRAME.emit_side.set_bits(side as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                x87::plus_int_times_80(
                    field(wheel, 0x3C + 4 * k as u32)?,
                    EMITTER.side_b[k].bits(),
                    side,
                )
            })
        })?;
        put(0x10, rounded(place[0]))?;
        put(0x14, rounded(place[1]))?;
        put(
            0x28,
            x87::ext_less(place[1], MARKS_FLOATS.two_pow_16.bits()),
        )?;
        put(0x18, rounded(place[2]))?;
        let motion: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k4 = 4 * k as u32;
                let f = [0, 1, 2].map(|i| KIND1_SET.motion_k[3 * k + i].bits());
                x87::three_products_80([
                    [matrix_cell(m, k4)?, field(wheel, 0x10)?, f[0]],
                    [matrix_cell(m, 0x20 + k4)?, field(wheel, 0x18)?, f[1]],
                    [EMITTER.side_b[k].bits(), field(wheel, 0xC)?, f[2]],
                ])
            })
        })?;
        let (moved, _) = jitter_point([
            extended(KIND1_SET.spread.bits()),
            motion[2],
            motion[1],
            motion[0],
        ]);
        put(0x34, rounded(moved[0]))?;
        put(0x30, x87::add(rounded(moved[1]), KIND1_SET.lift.bits()))?;
        put(0x2C, rounded(moved[2]))?;
        let r = next_c5(&KIND1_SET.spin_random);
        put(0x54, (r & 0x3F).wrapping_sub(0x20))?;
        put(0x50, r.rotate_right(6) & 0xFFF)?;
        put(0x58, flags & 0x3F)?;
        put(0x84, life as u32)?;
        put(0x78, x87::over_int(fade, life))?;
        put(0x74, fade)?;
        put(0x68, EMITTER.car.get())?;
        put(0x70, SIZE)?;
        put(0x6C, car.word(0x88)?)?;
        put(0x60, x87::int_times(SIZE as i32, KIND1_SET.size_k.bits()))?;
        put(0x64, 0)?;
    }
    Ok(())
}

/// The Rust version (see the header).
pub fn emit_kind_1(wheel: GamePtr, object: &'static MarkCar) -> Result<(), Refused> {
    if KIND1_CHOICE.choice.get() != 0 {
        emit_particle_21(wheel, object)
    } else {
        emit_particle_1(wheel, object)
    }
}

/// The wheel kind 1's choice (IDA 0x93A1D6 .. 0x93A1DA, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Kind1Choice {
    /// 0x93A1D6: `g_marksKind1Choice`: `emit_particle_21` when not 0 (`ring_frame` sets it from the rain's drops)
    pub choice: Dword,
}

/// Kind1Choice's first values.
pub static KIND1_CHOICE: Kind1Choice = Kind1Choice {
    choice: Dword::new(0x0),
};

impl Kind1Choice {
    pub const START: usize = 0x93A1D6;
    pub const LEN: usize = 0x4;
}

#[cfg(test)]
mod kind1_choice_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind1_choice_starts_as_the_games() {
        assert_eq!(size_of::<Kind1Choice>(), Kind1Choice::LEN);
        assert!(
            bytes(&KIND1_CHOICE) == crate::moved::image_bytes(Kind1Choice::START, Kind1Choice::LEN),
            "KIND1_CHOICE starts differently from the image"
        );
        assert_eq!(
            offset_of!(Kind1Choice, choice),
            0x93A1D6 - Kind1Choice::START
        );
    }
}

/// Kind 1's emitter variables (`emit_particle_1`) (IDA 0x93A39A .. 0x93A3D2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Kind1Set {
    /// 0x93A39A: `dword_93A39A`: the side's random
    pub side_random: Dword,
    /// 0x93A39E: `flt_93A39E` .. `flt_93A3BE`: per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [Float; 9],
    /// 0x93A3C2: `flt_93A3C2`: the spread `jitter_point` takes
    pub spread: Float,
    /// 0x93A3C6: `flt_93A3C6`: added to y
    pub lift: Float,
    /// 0x93A3CA: `dword_93A3CA`: the spin's random
    pub spin_random: Dword,
    /// 0x93A3CE: `flt_93A3CE`: the size's factor
    pub size_k: Float,
}

/// Kind1Set's first values.
pub static KIND1_SET: Kind1Set = Kind1Set {
    side_random: Dword::new(0x0),
    motion_k: [
        Float::from_bits(0x4283126F),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0x41426E98),
        Float::from_bits(0x4283126F),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0x41426E98),
        Float::from_bits(0x4283126F),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0x41426E98),
    ],
    spread: Float::from_bits(0x42C00000),
    lift: Float::from_bits(0x42800000),
    spin_random: Dword::new(0x0),
    size_k: Float::from_bits(0x3E4CCCCD),
};

impl Kind1Set {
    pub const START: usize = 0x93A39A;
    pub const LEN: usize = 0x38;
}

#[cfg(test)]
mod kind1_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind1_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind1Set>(), Kind1Set::LEN);
        assert!(
            bytes(&KIND1_SET) == crate::moved::image_bytes(Kind1Set::START, Kind1Set::LEN),
            "KIND1_SET starts differently from the image"
        );
        assert_eq!(
            offset_of!(Kind1Set, side_random),
            0x93A39A - Kind1Set::START
        );
        assert_eq!(offset_of!(Kind1Set, motion_k), 0x93A39E - Kind1Set::START);
        assert_eq!(offset_of!(Kind1Set, spread), 0x93A3C2 - Kind1Set::START);
        assert_eq!(offset_of!(Kind1Set, lift), 0x93A3C6 - Kind1Set::START);
        assert_eq!(
            offset_of!(Kind1Set, spin_random),
            0x93A3CA - Kind1Set::START
        );
        assert_eq!(offset_of!(Kind1Set, size_k), 0x93A3CE - Kind1Set::START);
    }
}
