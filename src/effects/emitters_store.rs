//! The particle emitters' variables.

use crate::memory::session::{PlainDword, PlainFloat, PlainInt};

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
    /// writes the car's `motion`).
    pub motion: [PlainFloat; 3],
    /// The looks by the car's model (`effects::cars_particles`), or from the first wheel record's
    /// place x and z (`effects::emit_car_particles`).
    pub looks: [PlainDword; 2],
    /// The car's number times `tag_k`; each ring entry's `emitter_tag`.
    pub tag: PlainDword,
    /// The six floats `effects::reset_settings_blocks` puts in each settings block's `motion` and
    /// first `steps` row.
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
