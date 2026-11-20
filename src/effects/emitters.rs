//! What the marks' particle emitters under `effects::emit_wheel_particles` share: the ring
//! of 0x94-byte entries (a cursor and a count, at most 0x3FE taken), the rate gate on a
//! wheel record and its object, and their randoms. `effects::emit_wheel_particles` picks an
//! emitter by the wheel record's +0x1C (the surface's particle kind).

use crate::effects::owner::Effects;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
use crate::memory::session::{PlainDword, PlainFloat, PlainInt};
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

/// The next entry taken (its number + 1; its +8 `EmitterVars.tag`, +0x90 `tag`, +0xC `kind`).
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
    effects: &Effects,
    wheel: &MarkRecord,
    object: &CarEffects,
    a: &impl crate::memory::session::Cell32,
    b: &impl crate::memory::session::Cell32,
    k: &[crate::memory::session::PlainFloat; 4],
) -> Result<Gate, Refused> {
    Ok({
        let [k1, k2, k3, k4] = k.each_ref().map(crate::memory::session::PlainFloat::get);
        let rate = x87::rate_to_int(a, b, k1, k2, k3, &effects.setup.floats.one, k4);
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
        // Moving unless both are +0 (the bits tested).
        let still = |v: f32| v == 0.0 && v.is_sign_positive();
        let moving = !still(wheel.local_velocity_x.get()) || !still(wheel.local_velocity_z.get());
        Gate {
            rate,
            count: Some(rest),
            passes: moving,
        }
    })
}

/// The particle emitters' variables: the record they are on, the vectors they share, and their
/// constants and randoms.
#[derive(rc2k_derive::Cells)]
pub struct EmitterVars {
    /// Which wheel record the emitters are on (0 .. 3, 2 or 3 for the body); bit 0 the side.
    pub record: PlainDword,
    /// -1 or 1 (`effects::emit_slip_spray`).
    pub sign: PlainDword,
    /// The car object being done (a `effects_settings_store::car_handle`, 0 for none).
    pub car: PlainDword,
    /// The second side vector of the emitters.
    pub side_b: [PlainFloat; 3],
    /// The up vector `effects::cars_particles` crosses with the car's.
    pub up: [PlainFloat; 3],
    /// The first cross product, made of length 1.
    pub across: [PlainFloat; 3],
    /// The first side vector of the emitters.
    pub side_a: [PlainFloat; 3],
    /// A motion vector (`effects::emit_body_contact_particles` reads it as floats, `emit_twin`
    /// writes the car's +0x4C ..).
    pub motion: [PlainFloat; 3],
    /// The looks by the car's model (`effects::cars_particles`), or from the car's +0x110 and
    /// +0x118 (`effects::emit_car_particles`).
    pub looks: [PlainDword; 2],
    /// The car's number times `tag_k`; each ring entry's +8.
    pub tag: PlainDword,
    /// The six floats `effects::reset_settings_blocks` puts in each settings block's +0x4C ..
    pub settings_start: [PlainFloat; 6],
    /// What a car's number is multiplied by for `tag` (an integer).
    pub tag_k: PlainInt,
    /// The car's rate: the mean's length times the first plus the second, one over it, times the
    /// third.
    pub rate_k: [PlainFloat; 3],
    /// The wheel places' factors and bases (`effects::emit_car_particles`).
    pub place_k: [PlainFloat; 7],
    /// The side factors for the body record (`effects::emit_car_particles`).
    pub body_k: [PlainFloat; 6],
    /// `effects::emit_wheel_particles`' random (times 0xC5 plus 0x5D) for the surface copy.
    pub wheel_random: PlainDword,
    /// `effects::emit_body_particles`' random for the surface copy.
    pub body_random: PlainDword,
    /// A random side (integers) the emitters' places are shifted by.
    pub sides: [PlainDword; 2],
    /// `effects::emit_slip_spray`'s strength.
    pub strength: PlainFloat,
}

impl EmitterVars {
    pub const fn new() -> Self {
        Self {
            record: PlainDword::new(0x0),
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
            settings_start: [const { PlainFloat::new(0.0) }; 6],
            tag_k: PlainInt::new(0x4C8),
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
            sides: [const { PlainDword::new(0x0) }; 2],
            strength: PlainFloat::new(0.0),
        }
    }
}
