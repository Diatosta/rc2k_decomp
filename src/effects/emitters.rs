//! What the marks' particle emitters under `effects::emit_wheel_particles` share: the ring
//! of 0x94-byte entries (a cursor and a count, at most 0x3FE taken), the rate gate on a
//! wheel record and its object, and their randoms. `effects::emit_wheel_particles` picks an
//! emitter by the wheel record's `kind` (the surface's particle kind).

use crate::effects::owner::Effects;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
use crate::memory::session::PlainDword;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};

pub use crate::effects::ring_store::Ring;

pub(super) const MOST_TAKEN: u32 = 0x3FE;

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
pub(super) fn next_c5(at: impl Random) -> u32 {
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

/// The next entry taken (its number + 1; its `emitter_tag` `EmitterVars.tag`, its `car_tag` `tag`,
/// its `kind` `kind`).
pub(super) fn take_entry(
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
/// cells.
pub(super) fn matrix_cell<C: crate::memory::session::Cell32>(
    matrix: &[C],
    offset: u32,
) -> Result<u32, Refused> {
    matrix
        .get(offset as usize / 4)
        .map(C::bits)
        .ok_or_else(|| BadInput("a car matrix's cell past its end").into())
}

/// As `matrix_cell`, the cell read as the float it holds.
pub(super) fn matrix_float<C: crate::memory::session::Cell32>(
    matrix: &[C],
    offset: u32,
) -> Result<f32, Refused> {
    matrix_cell(matrix, offset).map(f32::from_bits)
}

/// The car the emitters are on (`EmitterVars.car`, a handle); refused when none is set.
pub(super) fn emitting_car(
    effects: &crate::effects::owner::Effects,
) -> Result<&'static CarEffects, BadInput> {
    effects
        .car_record(effects.kinds.emitter.car.get())
        .ok_or(BadInput("no car for the emitters"))
}

/// Whether the emitters are on a car whose matrix is set (the emitters' refusal).
pub(super) fn car_with_matrix(effects: &Effects) -> Result<bool, Refused> {
    match effects.car_record(effects.kinds.emitter.car.get()) {
        Some(car) => Ok(effects.has_whole(car)),
        None => Ok(false),
    }
}

/// The rate gate's outcome, worked out without writing.
pub struct Gate {
    /// `RingWork.rate`.
    pub rate: i32,
    /// The wheel's new `pace` when the second test is reached.
    pub count: Option<u32>,
    /// Whether an entry is made (all tests passed and the wheel moves).
    pub passes: bool,
}

/// The gate: the rate (`x87::rate_to_int` of the wheel's `a` and `b` floats by `k`); the
/// object's `rate` plus it at least 0x1000; the wheel's `pace` plus the object's `pace` at
/// least 1.5 times that sum (both signed), then kept as its remainder by it; the wheel's
/// `local_velocity_x` or `local_velocity_z` not 0.
pub fn gate(
    effects: &Effects,
    wheel: &MarkRecord,
    object: &CarEffects,
    a: &impl crate::memory::session::Cell32,
    b: &impl crate::memory::session::Cell32,
    k: &[crate::memory::session::PlainFloat; 4],
) -> Result<Gate, Refused> {
    Ok({
        let [k1, k2, k3, k4] = k.each_ref().map(crate::memory::session::PlainFloat::get);
        let rate = x87::rate_to_int(a, b, k1, k2, k3, effects.setup.floats.one, k4);
        gate_with(wheel, object, rate)?
    })
}

/// The gate after the rate (see `gate`).
pub(super) fn gate_with(
    wheel: &MarkRecord,
    object: &CarEffects,
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
        // The dividend is the unsigned total.
        let rest = (total as i64 % limit as i32 as i64) as u32;
        // Moving unless both are plain zero (the bits tested).
        let still = |v: f32| v == 0.0 && v.is_sign_positive();
        let moving = !still(wheel.local_velocity_x.get()) || !still(wheel.local_velocity_z.get());
        Gate {
            rate,
            count: Some(rest),
            passes: moving,
        }
    })
}
