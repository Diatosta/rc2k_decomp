//! What the marks' particle emitters under `effects::emit_wheel_particles` share (IDA):
//! the ring of 0x94-byte entries (`g_marksRingCursor`, `g_marksRingCount`, at most 0x3FE taken),
//! the rate gate on a wheel record and its object, and their randoms.
//! `effects::emit_wheel_particles` (0x4CA085) picks an emitter by the wheel record's +0x1C
//! (the surface's particle kind); the patched EXE some owners use turns it into a `ret`
//! (section 16).

use crate::effects::owner::effects;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
use crate::session::{PlainBytes, PlainDword, PlainFloat};

pub use crate::effects::ring_store::Ring;

pub const MOST_TAKEN: u32 = 0x3FE;

/// A variable the emitters keep a random in (a typed dword, `&Dword`).
pub trait Random: Copy {
    fn load(self) -> u32;
    fn store(self, value: u32);
}

impl Random for &PlainDword {
    fn load(self) -> u32 {
        self.get()
    }
    fn store(self, value: u32) {
        self.set(value);
    }
}

/// A random of the emitters' kind: times 0xC5 plus 0x5D, kept.
pub fn next_c5(at: impl Random) -> u32 {
    let r = at.load().wrapping_mul(0xC5).wrapping_add(0x5D);
    at.store(r);
    r
}

/// The entry the next `take_entry` takes, if one is left.
pub fn next_entry(ring: &Ring) -> Result<Option<u32>, Refused> {
    Ok({
        if ring.work.count.get() == MOST_TAKEN {
            return Ok(None);
        }
        Some(crate::effects::ring_entries::entry(ring, ring.work.cursor.get())?.prev)
    })
}

/// The next entry taken (its number + 1; its +8 `dword_93547E`, +0x90 `tag`, +0xC `kind`).
pub fn take_entry(
    ring: &Ring,
    effects: &crate::effects::owner::Effects,
    tag: u32,
    kind: u32,
) -> Result<u32, Refused> {
    Ok({
        let e = crate::effects::ring_entries::entry(ring, ring.work.cursor.get())?.prev;
        ring.work.cursor.set(e);
        ring.work.count.set(ring.work.count.get().wrapping_add(1));
        let slot = crate::effects::ring_entries::Slot::new(ring, e);
        slot.put(|en| &mut en.emitter_tag, effects.kinds.emitter.tag.get())?;
        slot.put(|en| &mut en.car_tag, tag)?;
        slot.put(|en| &mut en.kind, kind)?;
        e
    })
}

/// The dword at byte `offset` of a car's matrix (16 cells) or matrices; refused past the
/// cells (the original reads on into the block).
pub fn matrix_cell<C: crate::session::Cell32>(matrix: &[C], offset: u32) -> Result<u32, Refused> {
    matrix
        .get(offset as usize / 4)
        .map(C::bits)
        .ok_or_else(|| BadInput("a car matrix's cell past its end").into())
}

/// The car the emitters are on (`EMITTER.car`, a handle); refused when none is set.
pub fn emitting_car(
    effects: &crate::effects::owner::Effects,
) -> Result<&'static CarEffects, BadInput> {
    crate::race::effects_settings_store::car_or_none(effects.kinds.emitter.car.get())
        .ok_or(BadInput("no car for the emitters"))
}

/// Whether the emitters are on a car whose matrix is set (the emitters' refusal).
pub fn car_with_matrix() -> Result<bool, Refused> {
    match crate::race::effects_settings_store::car_or_none(effects().kinds.emitter.car.get()) {
        Some(car) => Ok(car.has_whole()),
        None => Ok(false),
    }
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
    wheel: &MarkRecord,
    object: &'static CarEffects,
    a: &impl crate::session::Cell32,
    b: &impl crate::session::Cell32,
    k: &[crate::session::PlainFloat; 4],
) -> Result<Gate, Refused> {
    Ok({
        let [k1, k2, k3, k4] = k.each_ref().map(super::super::session::PlainFloat::bits);
        let rate = x87::rate_to_int(a, b, k1, k2, k3, &effects().setup.floats.one, k4);
        gate_with(wheel, object, rate)?
    })
}

/// The gate after the rate (see `gate`).
pub fn gate_with(
    wheel: &MarkRecord,
    object: &'static CarEffects,
    rate: i32,
) -> Result<Gate, Refused> {
    Ok({
        let sum = object.rate.get().wrapping_add(rate as u32);
        if (sum as i32) < 0x1000 {
            return Ok(Gate {
                rate,
                count: None,
                passes: false,
            });
        }
        let total = wheel.pace.get().wrapping_add(object.pace.get());
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
        let moving = wheel.local_velocity_x.bits() != 0 || wheel.local_velocity_z.bits() != 0;
        Gate {
            rate,
            count: Some(rest),
            passes: moving,
        }
    })
}

/// The particle emitters' variables (IDA 0x935416 .. 0x935562): the record they are on, the vectors they share, and their constants and randoms.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct EmitterVars {
    /// 0x935416: `g_marksRecordNumber` (`dword_935416`): which wheel record the emitters are on (0 .. 3, 2 or 3 for the body); bit 0 the side.
    pub record: PlainDword,
    /// 0x93541A: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_93541a: PlainBytes<24>,
    /// 0x935432: `g_marksSign`: -1 or 1 (`effects::emit_slip_spray`).
    pub sign: PlainDword,
    /// 0x935436: `g_marksCar`: the car object being done (a `effects_settings_store::car_handle`, 0 for none).
    pub car: PlainDword,
    /// 0x93543A: `g_marksSideB` (`flt_93543A`, `flt_93543E`, `flt_935442`): the second side vector of the emitters.
    pub side_b: [PlainFloat; 3],
    /// 0x935446: `flt_935446` .. `flt_93544E`: the up vector `effects::cars_particles` crosses with the car's.
    pub up: [PlainFloat; 3],
    /// 0x935452: `g_marksAcross` (`flt_935452` .. `flt_93545A`): the first cross product, made of length 1.
    pub across: [PlainFloat; 3],
    /// 0x93545E: `g_marksSideA` (`flt_93545E` .. `flt_935466`): the first side vector of the emitters.
    pub side_a: [PlainFloat; 3],
    /// 0x93546A: `flt_93546A` .. `flt_935472`: a motion vector (`effects::emit_body_contact_particles` reads it as floats, `emit_twin` writes the car's +0x4C ..).
    pub motion: [PlainFloat; 3],
    /// 0x935476: `dword_935476`, `dword_93547A`: the looks by the car's model (`effects::cars_particles`), or from the car's +0x110 and +0x118 (`effects::emit_car_particles`).
    pub looks: [PlainDword; 2],
    /// 0x93547E: `g_marksTag` (`dword_93547E`): the car's number times `tag_k`; each ring entry's +8.
    pub tag: PlainDword,
    /// 0x935482: Not named: 68 bytes (a gap, or unreferenced data).
    pub _gap_935482: PlainBytes<68>,
    /// 0x9354C6: `flt_9354C6` .. `flt_9354DA`: the six floats `effects::reset_settings_blocks` puts in each settings block's +0x4C ..
    pub settings_start: [PlainFloat; 6],
    /// 0x9354DE: `dword_9354DE`: what a car's number is multiplied by for `tag` (an integer).
    pub tag_k: PlainDword,
    /// 0x9354E2: `flt_9354E2`, `flt_9354E6`, `flt_9354EA`: the car's rate: the mean's length times the first plus the second, one over it, times the third.
    pub rate_k: [PlainFloat; 3],
    /// 0x9354EE: `flt_9354EE` .. `flt_935506`: the wheel places' factors and bases (`effects::emit_car_particles`).
    pub place_k: [PlainFloat; 7],
    /// 0x93550A: `flt_93550A` .. `flt_93551E`: the side factors for the body record (`effects::emit_car_particles`).
    pub body_k: [PlainFloat; 6],
    /// 0x935522: `dword_935522`: `effects::emit_wheel_particles`' random (times 0xC5 plus 0x5D) for the surface copy.
    pub wheel_random: PlainDword,
    /// 0x935526: `dword_935526`: `effects::emit_body_particles`' random for the surface copy.
    pub body_random: PlainDword,
    /// 0x93552A: Not named: 32 bytes (a gap, or unreferenced data).
    pub _gap_93552a: PlainBytes<32>,
    /// 0x93554A: `dword_93554A`, `dword_93554E`: a random side (integers) the emitters' places are shifted by.
    pub sides: [PlainDword; 2],
    /// 0x935552: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_935552: PlainBytes<12>,
    /// 0x93555E: `g_marksStrength` (`flt_93555E`): `effects::emit_slip_spray`'s strength.
    pub strength: PlainFloat,
}

impl EmitterVars {
    pub const fn new() -> Self {
        Self {
            record: PlainDword::new(0x0),
            _gap_93541a: PlainBytes::new([
                0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00,
                0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
            sign: PlainDword::new(0x0),
            car: PlainDword::new(0x0),
            side_b: [const { PlainFloat::new(0.0) }; 3],
            up: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            across: [const { PlainFloat::new(0.0) }; 3],
            side_a: [const { PlainFloat::new(0.0) }; 3],
            motion: [const { PlainFloat::new(0.0) }; 3],
            looks: [const { PlainDword::new(0x0) }; 2],
            tag: PlainDword::new(0x0),
            _gap_935482: PlainBytes::new([0; 68]),
            settings_start: [const { PlainFloat::new(0.0) }; 6],
            tag_k: PlainDword::new(0x4C8),
            rate_k: [
                PlainFloat::new(0.00002),
                PlainFloat::new(1.0),
                PlainFloat::new(196608.0),
            ],
            place_k: [
                PlainFloat::new(65.536),
                PlainFloat::new(-65.536),
                PlainFloat::new(65.536),
                PlainFloat::new(8192.0),
                PlainFloat::new(65.536),
                PlainFloat::new(20299.2),
                PlainFloat::new(45735.0),
            ],
            body_k: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(-1.0),
            ],
            wheel_random: PlainDword::new(0x0),
            body_random: PlainDword::new(0x0),
            _gap_93552a: PlainBytes::new([
                0x00, 0x00, 0x00, 0x43, 0x00, 0x00, 0x80, 0x42, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00,
                0x80, 0xC1, 0x00, 0x00, 0x80, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]),
            sides: [const { PlainDword::new(0x0) }; 2],
            _gap_935552: PlainBytes::new([0; 12]),
            strength: PlainFloat::new(0.0),
        }
    }
}

impl EmitterVars {
    pub const START: usize = 0x935416;
    pub const LEN: usize = 0x14C;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_emitters_vars_starts_as_the_games() {
        assert_eq!(size_of::<EmitterVars>(), EmitterVars::LEN);
        let p = (&raw const effects().kinds.emitter).cast::<u8>();
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
