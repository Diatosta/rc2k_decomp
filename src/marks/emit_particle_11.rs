//! `marks::emit_particle_11` (0x4CC35A, a chunk of `marks::emit_wheel_particles` entered
//! by `jz` from it and from 0x4CA215, left by `popa; retn`): one ring entry of kind 0xB;
//! `marks::emit_particle_5`'s code on its own variables (`dword_93A262`, `flt_93A266` ..
//! `flt_93A286`), a life of 2000, and made only when the wheel's |+0x18| rounded to an
//! integer (`fist`, the x87's rounding) is over 1 (kind 5: when it is not 0).
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! Refuses as `emit_particle_5`.

use crate::errors::Refused;
use crate::marks::emit_particle_5::{JitterSet, emit_particle_5_with};
use crate::session::{Bytes, Dword, Float};

/// Kind 0xB's variables.
pub static ELEVENTH: JitterSet = JitterSet {
    kind: 0xB,
    over_one: true,
    place_random: &KIND11_SET.place_random,
    motion_k: [
        &KIND11_SET.motion_x,
        &KIND11_SET.motion_y,
        &KIND11_SET.motion_z,
    ],
    lift: &KIND11_SET.lift,
    spread: &KIND11_SET.spread,
    roll_k: [
        &KIND11_SET.roll_k[0],
        &KIND11_SET.roll_k[1],
        &KIND11_SET.roll_k[2],
    ],
    spin: &KIND11_SET.spin,
    life: 0x7D0,
};

/// The Rust version (see the header).
pub unsafe fn emit_particle_11(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe { emit_particle_5_with(&ELEVENTH, wheel, object) }
}

/// Kind 0xB's emitter variables (`emit_particle_5::ELEVENTH`), and the constants of code that is in no function (IDA 0x93A262 .. 0x93A312, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct Kind11Set {
    /// 0x93A262: `dword_93A262`: the place's random
    pub place_random: Dword,
    /// 0x93A266: `flt_93A266`: the motion's x factor
    pub motion_x: Float,
    /// 0x93A26A: `flt_93A26A`: the motion's y factor
    pub motion_y: Float,
    /// 0x93A26E: `flt_93A26E`: added to the motion's y
    pub lift: Float,
    /// 0x93A272: `flt_93A272`: the motion's z factor
    pub motion_z: Float,
    /// 0x93A276: `flt_93A276`: the spread `jitter_point` takes
    pub spread: Float,
    /// 0x93A27A: `flt_93A27A` .. `flt_93A282`: the third row's factors (z, y, x)
    pub roll_k: [Float; 3],
    /// 0x93A286: `flt_93A286`: the spin's factor
    pub spin: Float,
    /// 0x93A28A: `flt_93A28E` .. `dword_93A30A`: read only by code at 0x4CC56C .. 0x4CC9BB, which is in no function (no ported code reads them)
    pub _gap_93a28a: Bytes<136>,
}

/// Kind11Set's first values.
pub static KIND11_SET: Kind11Set = Kind11Set {
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
    _gap_93a28a: Bytes::new([
        0x08, 0xAC, 0x82, 0x42, 0x0A, 0xD7, 0xA3, 0x3C, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x40,
        0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x47, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B,
        0x44, 0x42, 0xA6, 0x9B, 0x44, 0xC2, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0x42, 0xA6,
        0x9B, 0x44, 0xC2, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0xC2,
        0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C, 0x3E, 0x08, 0xAC, 0x82, 0x42, 0xAC, 0xC5, 0xA7,
        0x37, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x40, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0x47, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0xC2, 0xA6,
        0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0xC2, 0xA6, 0x9B, 0x44, 0x42,
        0xA6, 0x9B, 0x44, 0x42, 0xA6, 0x9B, 0x44, 0xC2, 0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C,
        0x3E,
    ]),
};

impl Kind11Set {
    pub const START: usize = 0x93A262;
    pub const LEN: usize = 0xB0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &KIND11_SET as *const Kind11Set as usize
    }
}

#[cfg(test)]
mod kind11_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind11_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind11Set>(), Kind11Set::LEN);
        assert!(
            bytes(&KIND11_SET) == crate::moved::image_bytes(Kind11Set::START, Kind11Set::LEN),
            "KIND11_SET starts differently from the image"
        );
        assert_eq!(
            offset_of!(Kind11Set, place_random),
            0x93A262 - Kind11Set::START
        );
        assert_eq!(offset_of!(Kind11Set, motion_x), 0x93A266 - Kind11Set::START);
        assert_eq!(offset_of!(Kind11Set, motion_y), 0x93A26A - Kind11Set::START);
        assert_eq!(offset_of!(Kind11Set, lift), 0x93A26E - Kind11Set::START);
        assert_eq!(offset_of!(Kind11Set, motion_z), 0x93A272 - Kind11Set::START);
        assert_eq!(offset_of!(Kind11Set, spread), 0x93A276 - Kind11Set::START);
        assert_eq!(offset_of!(Kind11Set, roll_k), 0x93A27A - Kind11Set::START);
        assert_eq!(offset_of!(Kind11Set, spin), 0x93A286 - Kind11Set::START);
    }
}
