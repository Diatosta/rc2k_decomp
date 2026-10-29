//! `effects::emit_particle_21` (0x4CCA58, a chunk of `effects::emit_wheel_particles` entered
//! by `jnz` from 0x4CCCDE when `dword_93A1D6` is not 0, left by `popa; retn`): one ring
//! entry of kind 0x15 for the wheel kind 1, when the rate gate passes.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The rate (`dword_9356B0`): (1 - 1 / (the wheel's +0xC by `flt_93A312`, `flt_93A316`,
//! plus `flt_93A31A`)) by `flt_93A31E`; then `marks/emitters.rs`' gate. The entry: its
//! place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the car matrix's first
//! row, 0 or 1 times its third), less `flt_93A326` in y at +0x28; its motion per axis the
//! matrix's first row by the wheel's +0x10, its third by +0x18 and by +0xC, each by a
//! factor (`flt_93A32A` ..); a random spin and turn (+0x54, +0x50); +0x58 0x34; a life of
//! 1000 (+0x78 `flt_7651A4` over it); +0x74 `dword_76519C`; the car (+0x68) and its +0x88,
//! +0x90 plus the rate; +0x60 that sum by `flt_93A352` (after a first value by the marks'
//! random, which still steps). Refuses (before anything) a null wheel, a null object, and a
//! null car or matrix when an entry is made.

use crate::effects::emitters::*;
use crate::effects::tuning::MARKS_FLOATS;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::scene::matrix3::rounded;
use crate::session::{Bytes, Dword, Float};

const LOOK: u32 = 0x34;
const LIFE: u32 = 0x3E8;

/// The rate, from the wheel's +0xC.
fn rate(wheel: u32) -> Result<i32, Refused> {
    Ok({
        let [k1, k2, k3, k4] = KIND21_SET.rate_k.each_ref().map(|f| f.bits());
        x87::speed_rate_to_int(at(wheel + 0xC)?, k1, k2, k3, MARKS_FLOATS.one.bits(), k4)
    })
}

/// Why the Rust version refuses these registers, if it does, worked out without writing.
fn refusal(wheel: u32, object: u32) -> Result<Option<&'static str>, Refused> {
    Ok({
        if wheel == 0 {
            return Ok(Some("a null wheel"));
        }
        if object == 0 {
            return Ok(Some("a null object"));
        }
        if !gate_with(wheel, object, rate(wheel)?)?.passes {
            return Ok(None);
        }
        // The ring full: nothing more is read.
        next_entry()?;
        let car: u32 = EMITTER.car.get();
        if car == 0 || at(car + 0x24)? == 0 {
            return Ok(Some("a null car object or matrix"));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_particle_21(wheel: u32, object: u32) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(wheel, object)? {
            return Err(BadInput(why).into());
        }
        let g = gate_with(wheel, object, rate(wheel)?)?;
        RING_WORK.rate.set(g.rate as u32);
        if let Some(count) = g.count {
            GamePtr::new(wheel + 4).write::<u32>(count, "a mark's field")?;
        }
        if !g.passes || next_entry()?.is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C)? & 0xFF_FFFF, 0x15)?;
        let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
        let car: u32 = EMITTER.car.get();
        let m = at(car + 0x24)?;
        let r = next_c5(&KIND21_SET.side_random);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        EMITTER.sides[0].set(a as u32);
        EMITTER.sides[1].set(b as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            let k = 4 * k as u32;
            Ok(x87::pair_sum_80(
                a,
                at(m + k)?,
                b,
                at(m + 0x20 + k)?,
                at(wheel + 0x3C + k)?,
            ))
        })?;
        put(0x18, rounded(place[2]))?;
        put(0x14, rounded(place[1]))?;
        put(0x28, x87::ext_less(place[1], KIND21_SET.lift.bits()))?;
        put(0x10, rounded(place[0]))?;
        let motion: [u32; 3] = crate::effects::try_from_fn(|k| {
            let (first, third) = (at(m + 4 * k as u32)?, at(m + 0x20 + 4 * k as u32)?);
            let f = [0, 1, 2].map(|i| KIND21_SET.motion_k[3 * k + i].bits());
            Ok(x87::three_products([
                [first, at(wheel + 0x10)?, f[0]],
                [third, at(wheel + 0x18)?, f[1]],
                [third, at(wheel + 0xC)?, f[2]],
            ]))
        })?;
        put(0x34, motion[2])?;
        put(0x30, motion[1])?;
        put(0x2C, motion[0])?;
        let r = next_c5(&KIND21_SET.spin_random);
        put(0x54, (r & 0x3F).wrapping_sub(0x20))?;
        put(0x50, r.rotate_right(6) & 0xFFF)?;
        put(0x58, LOOK)?;
        let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
        MARKS_WORK.seed.set(s);
        put(0x60, x87::from_int(((s & 0x3FFF) + 0x4000) as i32))?;
        put(0x84, LIFE)?;
        put(
            0x78,
            x87::over_int(MARKS_FLOATS.two_fifty_seven.bits(), LIFE as i32),
        )?;
        put(0x74, MARKS_FLOATS.two_fifty_five.bits())?;
        put(0x68, car)?;
        let size = at(car + 0x90)?.wrapping_add(g.rate as u32);
        put(0x70, size)?;
        put(0x6C, at(car + 0x88)?)?;
        put(0x60, x87::int_times(size as i32, KIND21_SET.size_k.bits()))?;
    }
    Ok(())
}

/// Kind 0x15's emitter variables (`emit_particle_21`) (IDA 0x93A312 .. 0x93A39A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct Kind21Set {
    /// 0x93A312: `flt_93A312` .. `flt_93A31E`: the rate's factors
    pub rate_k: [Float; 4],
    /// 0x93A322: `dword_93A322`: the side's random
    pub side_random: Dword,
    /// 0x93A326: `flt_93A326`: the place's drop in y
    pub lift: Float,
    /// 0x93A32A: `flt_93A32A` .. `flt_93A34A`: per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [Float; 9],
    /// 0x93A34E: `dword_93A34E`: the spin's random
    pub spin_random: Dword,
    /// 0x93A352: `flt_93A352`: the size's factor
    pub size_k: Float,
    /// 0x93A356: `flt_93A35E` .. `flt_93A396`: read only by code in no function
    pub _gap_93a356: Bytes<68>,
}

/// Kind21Set's first values.
pub static KIND21_SET: Kind21Set = Kind21Set {
    rate_k: [
        Float::from_bits(0x4282AC08),
        Float::from_bits(0x37A7C5AC),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x48400000),
    ],
    side_random: Dword::new(0x0),
    lift: Float::from_bits(0x47800000),
    motion_k: [
        Float::from_bits(0x42449BA6),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0xC2449BA6),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0xC2449BA6),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0x42449BA6),
        Float::from_bits(0xC2449BA6),
    ],
    spin_random: Dword::new(0x0),
    size_k: Float::from_bits(0x3E4CCCCD),
    _gap_93a356: Bytes::new([
        0x0A, 0xD7, 0x23, 0x45, 0x17, 0xB7, 0x51, 0x39, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x40,
        0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x47, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B,
        0x44, 0x42, 0x4C, 0x37, 0x99, 0xC1, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B, 0x44, 0x42, 0x4C,
        0x37, 0x99, 0xC1, 0x6F, 0x12, 0x83, 0x42, 0xA6, 0x9B, 0x44, 0x42, 0x4C, 0x37, 0x99, 0xC1,
        0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C, 0x3E,
    ]),
};

impl Kind21Set {
    pub const START: usize = 0x93A312;
    pub const LEN: usize = 0x88;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&KIND21_SET as *const Kind21Set) as usize
    }
}

#[cfg(test)]
mod kind21_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind21_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind21Set>(), Kind21Set::LEN);
        assert!(
            bytes(&KIND21_SET) == crate::moved::image_bytes(Kind21Set::START, Kind21Set::LEN),
            "KIND21_SET starts differently from the image"
        );
        assert_eq!(offset_of!(Kind21Set, rate_k), 0x93A312 - Kind21Set::START);
        assert_eq!(
            offset_of!(Kind21Set, side_random),
            0x93A322 - Kind21Set::START
        );
        assert_eq!(offset_of!(Kind21Set, lift), 0x93A326 - Kind21Set::START);
        assert_eq!(offset_of!(Kind21Set, motion_k), 0x93A32A - Kind21Set::START);
        assert_eq!(
            offset_of!(Kind21Set, spin_random),
            0x93A34E - Kind21Set::START
        );
        assert_eq!(offset_of!(Kind21Set, size_k), 0x93A352 - Kind21Set::START);
    }
}
