//! What the marks' particle emitters under `sub_4CA085` share (IDA, 2026-10-17): the ring
//! of 0x94-byte entries (`g_marksRingCursor`, `g_marksRingCount`, at most 0x3FE taken),
//! the rate gate on a wheel record (edi) and its object (esi), and their randoms.
//! `marks::emit_wheel_particles` (0x4CA085) picks an emitter by the wheel record's +0x1C
//! (the surface's particle kind); the patched EXE some owners use turns it into a `ret`
//! (section 16).

use crate::errors::Refused;
use crate::marks::tuning::MARKS_FLOATS;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::session::{Bytes, Dword, Float};

pub use crate::marks::ring_store::RING_WORK;

pub const MOST_TAKEN: u32 = 0x3FE;

/// A variable the emitters keep a random in (a typed dword, `&Dword`).
pub trait Random: Copy {
    fn load(self) -> u32;
    fn store(self, value: u32);
}

impl Random for &Dword {
    fn load(self) -> u32 {
        self.get()
    }
    fn store(self, value: u32) {
        self.set(value)
    }
}

/// A random of the emitters' kind: times 0xC5 plus 0x5D, kept.
pub fn next_c5(at: impl Random) -> u32 {
    let r = at.load().wrapping_mul(0xC5).wrapping_add(0x5D);
    at.store(r);
    r
}

/// The entry the next `take_entry` takes, if one is left.
pub fn next_entry() -> Result<Option<u32>, Refused> {
    Ok({
        if RING_WORK.count.get() == MOST_TAKEN {
            return Ok(None);
        }
        Some(GamePtr::new(RING_WORK.cursor.get()).read::<u32>("a mark's field")?)
    })
}

/// The next entry taken (its +8 `dword_93547E`, +0x90 `tag`, +0xC `kind`).
pub fn take_entry(tag: u32, kind: u32) -> Result<u32, Refused> {
    Ok({
        let e = GamePtr::new(RING_WORK.cursor.get()).read::<u32>("a mark's field")?;
        RING_WORK.cursor.set(e);
        RING_WORK.count.set(RING_WORK.count.get().wrapping_add(1));
        let put = |o: u32, v: u32| GamePtr::new(e + o).write::<u32>(v, "a mark's field");
        put(8, EMITTER.tag.get())?;
        put(0x90, tag)?;
        put(0xC, kind)?;
        e
    })
}

/// Reads a dword at an address.
pub fn at(a: u32) -> Result<u32, Refused> {
    Ok(GamePtr::new(a).read::<u32>("a mark's field")?)
}

/// The rate gate's outcome, worked out without writing.
pub struct Gate {
    /// `dword_9356B0`.
    pub rate: i32,
    /// The wheel's new +4 when the second test is reached.
    pub count: Option<u32>,
    /// Whether an entry is made (all tests passed and the wheel moves).
    pub passes: bool,
}

/// The gate: the rate (`x87::rate_to_int` of the wheel's `a` and `b` floats by `k`); the
/// object's +0x90 plus it at least 0x1000; the wheel's +4 plus the object's +0x88 at
/// least 1.5 times that sum (both signed), then kept as its remainder by it; the wheel's
/// +0x10 or +0x18 not 0.
pub fn gate(
    wheel: u32,
    object: u32,
    a: u32,
    b: u32,
    k: &[crate::session::Float; 4],
) -> Result<Gate, Refused> {
    Ok({
        let [k1, k2, k3, k4] = k.each_ref().map(|f| f.bits());
        let rate = x87::rate_to_int(
            at(wheel + a)?,
            at(wheel + b)?,
            k1,
            k2,
            k3,
            MARKS_FLOATS.one.bits(),
            k4,
        );
        gate_with(wheel, object, rate)?
    })
}

/// The gate after the rate (see `gate`).
pub fn gate_with(wheel: u32, object: u32, rate: i32) -> Result<Gate, Refused> {
    Ok({
        let sum = at(object + 0x90)?.wrapping_add(rate as u32);
        if (sum as i32) < 0x1000 {
            return Ok(Gate {
                rate,
                count: None,
                passes: false,
            });
        }
        let total = at(wheel + 4)?.wrapping_add(at(object + 0x88)?);
        let limit = sum.wrapping_add(sum).wrapping_sub(sum >> 1);
        if (total as i32) < limit as i32 {
            return Ok(Gate {
                rate,
                count: Some(total),
                passes: false,
            });
        }
        // `xor edx, edx; idiv`: the dividend the unsigned total.
        let rest = (total as i64 % limit as i32 as i64) as u32;
        let moving = at(wheel + 0x10)? != 0 || at(wheel + 0x18)? != 0;
        Gate {
            rate,
            count: Some(rest),
            passes: moving,
        }
    })
}

/// The particle emitters' variables (IDA 0x935416 .. 0x935562, the image's layout; phase 7, typed state): the record they are on, the vectors they share, and their constants and randoms.
#[repr(C)]
pub struct EmitterVars {
    /// 0x935416: `g_marksRecordNumber` (`dword_935416`): which wheel record the emitters are on (0 .. 3, 2 or 3 for the body); bit 0 the side.
    pub record: Dword,
    /// 0x93541A: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_93541a: Bytes<24>,
    /// 0x935432: `g_marksSign`: -1 or 1 (`marks::emit_particle_5`).
    pub sign: Dword,
    /// 0x935436: `g_marksCar`: the car object being done (an address).
    pub car: Dword,
    /// 0x93543A: `g_marksSideB` (`flt_93543A`, `flt_93543E`, `flt_935442`): the second side vector of the emitters.
    pub side_b: [Float; 3],
    /// 0x935446: `flt_935446` .. `flt_93544E`: the up vector `marks::cars_particles` crosses with the car's.
    pub up: [Float; 3],
    /// 0x935452: `g_marksAcross` (`flt_935452` .. `flt_93545A`): the first cross product, made of length 1.
    pub across: [Float; 3],
    /// 0x93545E: `g_marksSideA` (`flt_93545E` .. `flt_935466`): the first side vector of the emitters.
    pub side_a: [Float; 3],
    /// 0x93546A: `flt_93546A` .. `flt_935472`: a motion vector (`marks::emit_particle_18` reads it as floats, `emit_twin` writes the car's +0x4C ..).
    pub motion: [Float; 3],
    /// 0x935476: `dword_935476`, `dword_93547A`: the looks by the car's model (`marks::cars_particles`), or from the car's +0x110 and +0x118 (`marks::emit_car_particles`).
    pub looks: [Dword; 2],
    /// 0x93547E: `g_marksTag` (`dword_93547E`): the car's number times `tag_k`; each ring entry's +8.
    pub tag: Dword,
    /// 0x935482: Not named: 68 bytes (a gap, or unreferenced data).
    pub _gap_935482: Bytes<68>,
    /// 0x9354C6: `flt_9354C6` .. `flt_9354DA`: the six floats `marks::reset_settings_blocks` puts in each settings block's +0x4C ..
    pub settings_start: [Float; 6],
    /// 0x9354DE: `dword_9354DE`: what a car's number is multiplied by for `tag` (an integer).
    pub tag_k: Dword,
    /// 0x9354E2: `flt_9354E2`, `flt_9354E6`, `flt_9354EA`: the car's rate: the mean's length times the first plus the second, one over it, times the third.
    pub rate_k: [Float; 3],
    /// 0x9354EE: `flt_9354EE` .. `flt_935506`: the wheel places' factors and bases (`marks::emit_car_particles`).
    pub place_k: [Float; 7],
    /// 0x93550A: `flt_93550A` .. `flt_93551E`: the side factors for the body record (`marks::emit_car_particles`).
    pub body_k: [Float; 6],
    /// 0x935522: `dword_935522`: `marks::emit_wheel_particles`' random (times 0xC5 plus 0x5D) for the surface copy.
    pub wheel_random: Dword,
    /// 0x935526: `dword_935526`: `marks::emit_body_particles`' random for the surface copy.
    pub body_random: Dword,
    /// 0x93552A: Not named: 32 bytes (a gap, or unreferenced data).
    pub _gap_93552a: Bytes<32>,
    /// 0x93554A: `dword_93554A`, `dword_93554E`: a random side (integers) the emitters' places are shifted by.
    pub sides: [Dword; 2],
    /// 0x935552: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_935552: Bytes<12>,
    /// 0x93555E: `g_marksStrength` (`flt_93555E`): `marks::emit_particle_5`'s strength.
    pub strength: Float,
}

pub static EMITTER: EmitterVars = EmitterVars {
    record: Dword::new(0x0),
    _gap_93541a: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x80,
        0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
    sign: Dword::new(0x0),
    car: Dword::new(0x0),
    side_b: [const { Float::from_bits(0x0) }; 3],
    up: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    across: [const { Float::from_bits(0x0) }; 3],
    side_a: [const { Float::from_bits(0x0) }; 3],
    motion: [const { Float::from_bits(0x0) }; 3],
    looks: [const { Dword::new(0x0) }; 2],
    tag: Dword::new(0x0),
    _gap_935482: Bytes::new([0; 68]),
    settings_start: [const { Float::from_bits(0x0) }; 6],
    tag_k: Dword::new(0x4C8),
    rate_k: [
        Float::from_bits(0x37A7C5AC),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x48400000),
    ],
    place_k: [
        Float::from_bits(0x4283126F),
        Float::from_bits(0xC283126F),
        Float::from_bits(0x4283126F),
        Float::from_bits(0x46000000),
        Float::from_bits(0x4283126F),
        Float::from_bits(0x469E9666),
        Float::from_bits(0x4732A700),
    ],
    body_k: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0xBF800000),
    ],
    wheel_random: Dword::new(0x0),
    body_random: Dword::new(0x0),
    _gap_93552a: Bytes::new([
        0x00, 0x00, 0x00, 0x43, 0x00, 0x00, 0x80, 0x42, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00, 0x80,
        0xC1, 0x00, 0x00, 0x80, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ]),
    sides: [const { Dword::new(0x0) }; 2],
    _gap_935552: Bytes::new([0; 12]),
    strength: Float::from_bits(0x0),
};

impl EmitterVars {
    pub const START: usize = 0x935416;
    pub const LEN: usize = 0x14C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&EMITTER as *const EmitterVars) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_emitters_vars_starts_as_the_games() {
        assert_eq!(size_of::<EmitterVars>(), EmitterVars::LEN);
        let p = &EMITTER as *const EmitterVars as *const u8;
        let bytes: Vec<u8> = (0..size_of::<EmitterVars>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            bytes == crate::moved::image_bytes(EmitterVars::START, EmitterVars::LEN),
            "EMITTER starts differently from the image"
        );
        assert_eq!(
            offset_of!(EmitterVars, record),
            0x935416 - EmitterVars::START
        );
        assert_eq!(offset_of!(EmitterVars, sign), 0x935432 - EmitterVars::START);
        assert_eq!(offset_of!(EmitterVars, car), 0x935436 - EmitterVars::START);
        assert_eq!(
            offset_of!(EmitterVars, side_b),
            0x93543A - EmitterVars::START
        );
        assert_eq!(offset_of!(EmitterVars, up), 0x935446 - EmitterVars::START);
        assert_eq!(
            offset_of!(EmitterVars, across),
            0x935452 - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, side_a),
            0x93545E - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, motion),
            0x93546A - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, looks),
            0x935476 - EmitterVars::START
        );
        assert_eq!(offset_of!(EmitterVars, tag), 0x93547E - EmitterVars::START);
        assert_eq!(
            offset_of!(EmitterVars, settings_start),
            0x9354C6 - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, tag_k),
            0x9354DE - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, rate_k),
            0x9354E2 - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, place_k),
            0x9354EE - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, body_k),
            0x93550A - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, wheel_random),
            0x935522 - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, body_random),
            0x935526 - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, sides),
            0x93554A - EmitterVars::START
        );
        assert_eq!(
            offset_of!(EmitterVars, strength),
            0x93555E - EmitterVars::START
        );
    }
}
