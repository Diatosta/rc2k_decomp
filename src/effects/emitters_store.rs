//! The particle emitters' variables.

/// The particle emitters' variables: the record they are on, the vectors they share, and their
/// constants and randoms.
pub struct EmitterVars {
    /// Which wheel record the emitters are on (0 .. 3, 2 or 3 for the body); bit 0 the side.
    pub record: u32,
    /// -1 or 1 (`effects::emit_slip_spray`).
    pub sign: u32,
    /// The car object being done (a `effects_settings_store::car_handle`, 0 for none).
    pub car: u32,
    /// The second side vector of the emitters.
    pub side_b: [f32; 3],
    /// The up vector `effects::cars_particles` crosses with the car's.
    pub up: [f32; 3],
    /// The first cross product, made of length 1.
    pub across: [f32; 3],
    /// The first side vector of the emitters.
    pub side_a: [f32; 3],
    /// A motion vector (`effects::emit_body_contact_particles` reads it as floats, `emit_twin`
    /// writes the car's `motion`).
    pub motion: [f32; 3],
    /// The looks by the car's model (`effects::cars_particles`), or from the first wheel record's
    /// place x and z (`effects::emit_car_particles`).
    pub looks: [u32; 2],
    /// The car's number times `tag_k`; each ring entry's `emitter_tag`.
    pub tag: u32,
    /// The six floats `effects::reset_settings_blocks` puts in each settings block's `motion` and
    /// first `steps` row.
    pub settings_start: [f32; 6],
    /// What a car's number is multiplied by for `tag` (an integer).
    pub tag_k: i32,
    /// The car's rate: the mean's length times the first plus the second, one over it, times the
    /// third.
    pub rate_k: [f32; 3],
    /// The wheel places' factors and bases (`effects::emit_car_particles`).
    pub place_k: [f32; 7],
    /// The side factors for the body record (`effects::emit_car_particles`).
    pub body_k: [f32; 6],
    /// `effects::emit_wheel_particles`' random (times 0xC5 plus 0x5D) for the surface copy.
    pub wheel_random: u32,
    /// `effects::emit_body_particles`' random for the surface copy.
    pub body_random: u32,
    /// A random side (integers) the emitters' places are shifted by.
    pub sides: [u32; 2],
    /// `effects::emit_slip_spray`'s strength.
    pub strength: f32,
}

impl EmitterVars {
    pub const fn new() -> Self {
        Self {
            record: 0x0,
            sign: 0x0,
            car: 0x0,
            side_b: [0.0; 3],
            up: [0.0, 0.0, 1.0],
            across: [0.0; 3],
            side_a: [0.0; 3],
            motion: [0.0; 3],
            looks: [0x0; 2],
            tag: 0x0,
            settings_start: [0.0; 6],
            tag_k: 0x4C8,
            rate_k: [0.00002, 1.0, 196608.0],
            place_k: [65.536, -65.536, 65.536, 8192.0, 65.536, 20299.2, 45735.0],
            body_k: [1.0, 0.0, 0.0, 0.0, 0.0, -1.0],
            wheel_random: 0x0,
            body_random: 0x0,
            sides: [0x0; 2],
            strength: 0.0,
        }
    }
}
