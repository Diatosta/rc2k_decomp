//! `effects::emit_particle_13` (0x4CE05E): one ring entry of kind 0xD for a car each frame,
//! at its model's point (`flt_769312` .., 12 bytes by the car's +4).
//!
//! In:  esi the car object
//! Out: every register kept (`pusha` .. `popa`)
//!
//! The entry: its place the point (x negated; y less `flt_7651E4`; by `flt_7651D4`) turned
//! by `g_marksCar`'s matrix (placed and the place taken off again, on the x87 stack:
//! `x87::turned_place_less`), less `flt_93A462` in y at +0x28; the strength
//! (`dword_9356C0`, a float) 1 - 1 / (the car's +8 by `flt_93A466` plus 1); its motion the
//! matrix's third row by the strength and `flt_93A46A` ..; then its place moved by the
//! motion times a random's low word by `flt_765214` times `g_marksFrameTicks`; a random
//! spin and turn (+0x54, +0x50); +0x58 0x3F; a life of 250 (+0x78 the strength by
//! `flt_7651A4` over it); +0x74 the strength by `dword_7651A0`; +0x68 `g_marksCar` and its
//! +0x88; +0x70 `flt_93A47E` by the strength plus `flt_93A482`, rounded, and +0x60 that by
//! `flt_93A486`. Caller: `sub_4C9AF6` (the cars' loop). Refuses (before anything) a null
//! object, a null `g_marksCar` or matrix, and a model (+4) past the table's 34 entries,
//! when an entry is made.

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::*;
use crate::effects::setup::CLOCK;
use crate::effects::splines::SPLINE_HEAD;
use crate::effects::tuning::MARKS_FLOATS;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::MarkCar;
use crate::session::{Dword, Float, read_first};

const MODELS: u32 = 34;
const LOOK: u32 = 0x3F;
const LIFE: u32 = 0xFA;

/// Why the Rust version refuses, if it does, worked out without writing.
fn refusal(object: &'static MarkCar) -> Result<Option<&'static str>, Refused> {
    Ok({
        // The ring full: nothing more is read.
        next_entry()?;
        if !car_with_matrix()? {
            return Ok(Some("a null g_marksCar or matrix"));
        }
        if object.word(4)? >= MODELS {
            return Ok(Some("a car model past flt_769312's 34 entries"));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_particle_13(object: &'static MarkCar) -> Result<(), Refused> {
    if let Some(why) = refusal(object)? {
        return Err(BadInput(why).into());
    }
    if next_entry()?.is_none() {
        return Ok(());
    }
    let model = object.word(4)? as usize;
    let e = take_entry(object.word(0x1C)? & 0xFF_FFFF, 0xD)?;
    let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
    let car = emitting_car()?;
    let m = car.whole_matrix()?;
    let t: [u32; 3] = read_first(&SPLINE_HEAD.points[3 * model..]);
    let [x, y, y_less, z] = x87::turned_place_less(
        t,
        MARKS_FLOATS.plus_65_536.bits(),
        MARKS_FLOATS.seven_hundred.bits(),
        KIND13_SET.lift.bits(),
        m,
    );
    put(0x10, x)?;
    put(0x14, y)?;
    put(0x28, y_less)?;
    put(0x18, z)?;
    let one: u32 = MARKS_FLOATS.one.bits();
    let s = x87::one_less_inverse(object.word(8)?, KIND13_SET.strength_k.bits(), one);
    RING_WORK.whole.set(s);
    let motion: [u32; 3] = crate::effects::try_from_fn(|k| {
        Ok({
            x87::mul2(
                at(m + 0x20 + 4 * k as u32)?,
                s,
                KIND13_SET.motion_k[k].bits(),
            )
        })
    })?;
    put(0x2C, motion[0])?;
    put(0x30, motion[1])?;
    put(0x34, motion[2])?;
    let r = (next_c5(&KIND13_SET.drift_random) & 0xFFFF) as i32;
    let moved = x87::drifted(
        r,
        MARKS_FLOATS.over_65536.bits(),
        motion,
        CLOCK.frame_ticks.get() as i32,
        [x, y, z],
    );
    put(0x10, moved[0])?;
    put(0x14, moved[1])?;
    put(0x18, moved[2])?;
    let r = next_c5(&KIND13_SET.spin_random);
    put(0x54, (r & 0x3F).wrapping_sub(0x20))?;
    put(0x50, r.rotate_right(6) & 0xFFF)?;
    put(0x58, LOOK)?;
    put(0x84, LIFE)?;
    put(
        0x78,
        x87::times_over_int(s, MARKS_FLOATS.two_fifty_seven.bits(), LIFE as i32),
    )?;
    put(0x74, x87::mul(s, MARKS_FLOATS.two_fifty_six.bits()))?;
    put(0x68, EMITTER.car.get())?;
    let size = x87::times_plus_to_int(KIND13_SET.size_k[0].bits(), s, KIND13_SET.size_k[1].bits());
    put(0x70, size as u32)?;
    put(0x6C, car.word(0x88)?)?;
    put(0x60, x87::int_times(size, KIND13_SET.size_k[2].bits()))?;
    Ok(())
}

/// Kind 0xD's emitter variables (`emit_particle_13`) (IDA 0x93A462 .. 0x93A48A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct Kind13Set {
    /// 0x93A462: `flt_93A462`: the place's lift
    pub lift: Float,
    /// 0x93A466: `flt_93A466`: the strength's factor
    pub strength_k: Float,
    /// 0x93A46A: `flt_93A46A` .. `flt_93A472`: the motion's factors (x, y, z)
    pub motion_k: [Float; 3],
    /// 0x93A476: `dword_93A476`: the drift's random
    pub drift_random: Dword,
    /// 0x93A47A: `dword_93A47A`: the spin's random
    pub spin_random: Dword,
    /// 0x93A47E: `flt_93A47E`, `flt_93A482`, `flt_93A486`: the size's factor, base and scale
    pub size_k: [Float; 3],
}

/// Kind13Set's first values.
pub static KIND13_SET: Kind13Set = Kind13Set {
    lift: Float::from_bits(0x47800000),
    strength_k: Float::from_bits(0x38D1B717),
    motion_k: [
        Float::from_bits(0xC5000000),
        Float::from_bits(0xC5000000),
        Float::from_bits(0xC5000000),
    ],
    drift_random: Dword::new(0x0),
    spin_random: Dword::new(0x0),
    size_k: [
        Float::from_bits(0x47800000),
        Float::from_bits(0x45000000),
        Float::from_bits(0x3E4CCCCD),
    ],
};

impl Kind13Set {
    pub const START: usize = 0x93A462;
    pub const LEN: usize = 0x28;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const KIND13_SET) as usize
    }
}

#[cfg(test)]
mod kind13_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind13_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind13Set>(), Kind13Set::LEN);
        assert!(
            bytes(&KIND13_SET) == crate::moved::image_bytes(Kind13Set::START, Kind13Set::LEN),
            "KIND13_SET starts differently from the image"
        );
        assert_eq!(offset_of!(Kind13Set, lift), 0x93A462 - Kind13Set::START);
        assert_eq!(
            offset_of!(Kind13Set, strength_k),
            0x93A466 - Kind13Set::START
        );
        assert_eq!(offset_of!(Kind13Set, motion_k), 0x93A46A - Kind13Set::START);
        assert_eq!(
            offset_of!(Kind13Set, drift_random),
            0x93A476 - Kind13Set::START
        );
        assert_eq!(
            offset_of!(Kind13Set, spin_random),
            0x93A47A - Kind13Set::START
        );
        assert_eq!(offset_of!(Kind13Set, size_k), 0x93A47E - Kind13Set::START);
    }
}
