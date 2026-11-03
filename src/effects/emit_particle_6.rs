//! `effects::emit_particle_6` (0x4CDC74): one ring entry of kind 6 from a wheel, when its
//! flags (+0x20, kept in `dword_939BAE`) have 0xC0 and the rate gate passes
//! (`marks/emitters.rs`, the wheel's +0xC less +0x18 by `flt_93A41A` ..).
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The entry: its place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the
//! object's matrix's first row, 0 or 1 times its third) less 0x93A42E in y at +0x28; its
//! motion three sums of products of the wheel's speeds; a random spin and turn (+0x54,
//! +0x50), the flags' low 6 bits (+0x58), a size by the marks' random (+0x60, then
//! replaced), a life from `dword_939B76` by the flags' bits 6 and 7 (+0x84; +0x78
//! `flt_7651A4` over it), +0x74 `dword_76519C`, the object (+0x68) and its +0x88, +0x90.
//! Callers: `effects::emit_wheel_particles`' kinds 2 .. 4, 6 .. 9, 0xC (after `sub_4CD21A` or
//! `sub_4CD747`). Refuses (before anything) a null wheel, object or matrix where read, and
//! flags of 0x100 or more when an entry is made (the life read past `dword_939B76`'s four).

use crate::effects::emitters::car_with_matrix;
use crate::effects::emitters::emitting_car;
use crate::effects::emitters::field;
use crate::effects::emitters::*;
use crate::effects::surface_store::SURFACE;
use crate::effects::tuning::MARKS_FLOATS;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::marks_settings_store::MarkCar;
use crate::scene::matrix3::rounded;
use crate::session::{Dword, Float};

/// The motion: per axis (x, y, z) the three (factor, wheel speed) pairs; the scales are
/// `KIND6_SET.motion_k[3 * axis + n]`.
const MOTION: [[(&Float, u32); 3]; 3] = [
    [
        (&EMITTER.side_a[0], 0x10),
        (&EMITTER.side_b[0], 0x18),
        (&EMITTER.side_b[0], 0x0C),
    ],
    [
        (&EMITTER.side_a[1], 0x10),
        (&EMITTER.side_b[1], 0x18),
        (&EMITTER.side_b[1], 0x0C),
    ],
    [
        (&EMITTER.side_a[2], 0x10),
        (&EMITTER.side_b[2], 0x18),
        (&EMITTER.side_b[2], 0x0C),
    ],
];

/// Why the Rust version refuses these registers, if it does, worked out without writing.
fn refusal(wheel: GamePtr, object: &'static MarkCar) -> Result<Option<&'static str>, Refused> {
    Ok({
        let flags = field(wheel, 0x20)?;
        if flags & 0xC0 == 0 {
            return Ok(None);
        }
        if !gate(wheel, object, 0xC, 0x18, &KIND6_SET.rate_k)?.passes || next_entry()?.is_none() {
            return Ok(None);
        }
        if !car_with_matrix()? {
            return Ok(Some("a null car object or matrix"));
        }
        if flags >> 6 > 3 {
            return Ok(Some(
                "wheel flags of 0x100 or more: the life read past dword_939B76",
            ));
        }
        None
    })
}

/// The Rust version (see the header).
pub fn emit_particle_6(wheel: GamePtr, object: &'static MarkCar) -> Result<(), Refused> {
    {
        if let Some(why) = refusal(wheel, object)? {
            return Err(BadInput(why).into());
        }
        let flags = field(wheel, 0x20)?;
        SURFACE.wheel_flags.set(flags);
        if flags & 0xC0 == 0 {
            return Ok(());
        }
        let g = gate(wheel, object, 0xC, 0x18, &KIND6_SET.rate_k)?;
        RING_WORK.rate.set(g.rate as u32);
        if let Some(count) = g.count {
            wheel.offset(4).write::<u32>(count, "a mark's field")?;
        }
        if !g.passes || next_entry()?.is_none() {
            return Ok(());
        }
        let e = take_entry(object.word(0x1C)? & 0xFF_FFFF, 6)?;
        let put = |o: u32, v: u32| crate::effects::ring_entries::put(e, o, v);
        let car = emitting_car()?;
        let m = car.whole_matrix()?;
        let r = next_c5(&KIND6_SET.side_random);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        EMITTER.sides[0].set(a as u32);
        EMITTER.sides[1].set(b as u32);
        let place: [[u8; 10]; 3] = crate::effects::try_from_fn(|k| {
            Ok({
                let k = 4 * k as u32;
                x87::pair_sum_80(a, at(m + k)?, b, at(m + 0x20 + k)?, field(wheel, 0x3C + k)?)
            })
        })?;
        put(0x18, rounded(place[2]))?;
        put(0x14, rounded(place[1]))?;
        put(0x28, x87::ext_less(place[1], KIND6_SET.lift.bits()))?;
        put(0x10, rounded(place[0]))?;
        let motion: [u32; 3] = crate::effects::try_from_fn(|a| {
            Ok({
                x87::three_products(crate::effects::try_from_fn(|n| {
                    Ok({
                        let (f, s) = MOTION[a][n];
                        [
                            f.bits(),
                            field(wheel, s)?,
                            KIND6_SET.motion_k[3 * a + n].bits(),
                        ]
                    })
                })?)
            })
        })?;
        put(0x34, motion[2])?;
        put(0x30, motion[1])?;
        put(0x2C, motion[0])?;
        let r = next_c5(&KIND6_SET.spin_random);
        put(0x54, (r & 0x3F).wrapping_sub(0x20))?;
        put(0x50, r.rotate_right(6) & 0xFFF)?;
        put(0x58, flags & 0x3F)?;
        let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
        MARKS_WORK.seed.set(s);
        put(0x60, x87::from_int(((s & 0x3FFF) + 0x4000) as i32))?;
        let life: u32 = SURFACE.lives_by_flags[(flags >> 6) as usize].get();
        put(0x84, life)?;
        put(
            0x78,
            x87::over_int(MARKS_FLOATS.two_fifty_seven.bits(), life as i32),
        )?;
        put(0x74, MARKS_FLOATS.two_fifty_five.bits())?;
        put(0x68, EMITTER.car.get())?;
        let size = car.word(0x90)?.wrapping_add(g.rate as u32);
        put(0x70, size)?;
        put(0x6C, car.word(0x88)?)?;
        put(
            0x60,
            x87::int_times_plus(
                size as i32,
                KIND6_SET.size_k[0].bits(),
                KIND6_SET.size_k[1].bits(),
            ),
        )?;
    }
    Ok(())
}

/// Kind 6's emitter variables (`emit_particle_6`) (IDA 0x93A41A .. 0x93A462, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct Kind6Set {
    /// 0x93A41A: `flt_93A41A` .. `flt_93A426`: the rate's factors
    pub rate_k: [Float; 4],
    /// 0x93A42A: `dword_93A42A`: the side's random
    pub side_random: Dword,
    /// 0x93A42E: `flt_93A42E`: the place's drop in y
    pub lift: Float,
    /// 0x93A432: `flt_93A432` .. `flt_93A452`: per axis (x, y, z) the scales of the three products (wheel +0x10, +0x18, +0xC)
    pub motion_k: [Float; 9],
    /// 0x93A456: `dword_93A456`: the spin's random
    pub spin_random: Dword,
    /// 0x93A45A: `flt_93A45A`, `flt_93A45E`: the size's factors
    pub size_k: [Float; 2],
}

/// Kind6Set's first values.
pub static KIND6_SET: Kind6Set = Kind6Set {
    rate_k: [
        Float::from_bits(0x4282AC08),
        Float::from_bits(0x3951B717),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x48400000),
    ],
    side_random: Dword::new(0x0),
    lift: Float::from_bits(0x47800000),
    motion_k: [
        Float::from_bits(0x4283126F),
        Float::from_bits(0xC2449BA6),
        Float::from_bits(0x41D1374C),
        Float::from_bits(0x4283126F),
        Float::from_bits(0xC2449BA6),
        Float::from_bits(0x41D1374C),
        Float::from_bits(0x4283126F),
        Float::from_bits(0xC2449BA6),
        Float::from_bits(0x41D1374C),
    ],
    spin_random: Dword::new(0x0),
    size_k: [Float::from_bits(0x3E99999A), Float::from_bits(0x46000000)],
};

impl Kind6Set {
    pub const START: usize = 0x93A41A;
    pub const LEN: usize = 0x48;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const KIND6_SET) as usize
    }
}

#[cfg(test)]
mod kind6_set_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn kind6_set_starts_as_the_games() {
        assert_eq!(size_of::<Kind6Set>(), Kind6Set::LEN);
        assert!(
            bytes(&KIND6_SET) == crate::moved::image_bytes(Kind6Set::START, Kind6Set::LEN),
            "KIND6_SET starts differently from the image"
        );
        assert_eq!(offset_of!(Kind6Set, rate_k), 0x93A41A - Kind6Set::START);
        assert_eq!(
            offset_of!(Kind6Set, side_random),
            0x93A42A - Kind6Set::START
        );
        assert_eq!(offset_of!(Kind6Set, lift), 0x93A42E - Kind6Set::START);
        assert_eq!(offset_of!(Kind6Set, motion_k), 0x93A432 - Kind6Set::START);
        assert_eq!(
            offset_of!(Kind6Set, spin_random),
            0x93A456 - Kind6Set::START
        );
        assert_eq!(offset_of!(Kind6Set, size_k), 0x93A45A - Kind6Set::START);
    }
}
