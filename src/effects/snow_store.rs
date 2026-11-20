//! The snow's tables: `SnowTables` (the flakes' randoms, remainders and spreads) and `SnowCurve`
//! (the curve and its copy), `SnowState` (the flakes' variables, with the 16-byte constants),
//! `SnowCurveVars` (the walk's variables and the curve records) and `SnowLight` (the light, the
//! fade colours and the constants).
use crate::memory::session::PlainDword;

/// The snow's random tables (0x800 flakes).
pub struct SnowTables {
    /// The flakes' places, 20-bit randoms (`effects::make_noise_curve`).
    pub randoms: [[u32; 2048]; 3],
    /// Remainders of randoms (the curve indices' offsets for x and z).
    pub remainders: [[u32; 2048]; 2],
    /// The flakes' texture u (three bits of the seed times 0.0625 plus a base).
    pub spreads: [f32; 2048],
}

impl SnowTables {
    pub const fn new() -> Self {
        SnowTables {
            randoms: [[0; 2048]; 3],
            remainders: [[0; 2048]; 2],
            spreads: [0.0; 2048],
        }
    }
}

/// The snow's curve.
#[derive(rc2k_derive::Cells)]
pub struct SnowCurve {
    /// 0x1000 whole numbers, a smoothed random walk the flakes' x and z are moved by
    /// (`effects::make_noise_curve`).
    pub curve: [PlainDword; 4096],
    /// The curve copied while it is smoothed, then the curve less a ramp.
    pub copy: [PlainDword; 4096],
}

impl SnowCurve {
    pub const fn new() -> Self {
        SnowCurve {
            curve: [const { PlainDword::new(0x0) }; 4096],
            copy: [const { PlainDword::new(0x0) }; 4096],
        }
    }
}

/// The snow's variables.
pub struct SnowState {
    /// The turn's nine floats, each four times (`effects::run_snow`).
    pub wide_turn: [[f32; 4]; 9],
    /// Cleared by `effects::place_snow_view`.
    pub cleared: [u32; 8],
    /// The view point's z, four times (`effects::place_snow_view`).
    pub filled: [f32; 4],
    /// The focal length four times.
    pub wide_focal: [f32; 4],
    /// A constant loaded four times (`effects::make_noise_curve`).
    pub quad_a: [f32; 4],
    /// One over the view's farthest depth, four times (and `inverse` again).
    pub inverses: [f32; 4],
    /// A constant four times.
    pub quad_b: [f32; 4],
    /// A constant four times.
    pub quad_c: [f32; 4],
    /// -1.0 four times (nothing reads it).
    pub minus_ones: [f32; 4],
    /// The scene clock at the last snow frame.
    pub last_clock: u32,
    /// 0 with no flakes (and after `effects::make_noise_curve`).
    pub flag: u32,
    /// The settings' `listener_place`, the view's place.
    pub view_place: [u32; 3],
    /// The settings' `listener_turn`, the view's 3 x 3 matrix.
    pub view_matrix: [f32; 9],
    /// How far the flakes have fallen (28 bits).
    pub fall: u32,
    /// The flakes drawn this frame.
    pub drawn: u32,
    /// A copy of `flakes`.
    pub count_copy: u32,
    /// The flakes (the settings' `snow_flakes`, held to 0x800; initially 0x800).
    pub flakes: u32,
    /// The flake's depth as a whole number.
    pub depth: u32,
    /// One over the view's farthest depth.
    pub inverse: f32,
    /// The point `effects::place_snow_view` turns (whole numbers).
    pub point: [u32; 3],
    /// The view's place less the point (`effects::place_snow_view`).
    pub view: [u32; 3],
    /// The view matrix copied (`effects::place_snow_view`).
    pub matrix_copy: [f32; 9],
    /// The flake's place turned.
    pub turned: [f32; 3],
}

impl SnowState {
    pub const fn new() -> Self {
        SnowState {
            wide_turn: [[0.0, 0.0, 0.0, 0.0]; 9],
            cleared: [0x0; 8],
            filled: [0.0, 0.0, 0.0, 0.0],
            wide_focal: [0.0, 0.0, 0.0, 0.0],
            quad_a: [0.0, 0.0, 0.0, 0.0],
            inverses: [0.0, 0.0, 0.0, 0.0],
            quad_b: [0.0, 0.0, 0.0, 0.0],
            quad_c: [0.0, 0.0, 0.0, 0.0],
            minus_ones: [-1.0, -1.0, -1.0, -1.0],
            last_clock: 0x0,
            flag: 0x0,
            view_place: [0x0, 0x0, 0x0],
            view_matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            fall: 0x0,
            drawn: 0x0,
            count_copy: 0x0,
            flakes: 0x800,
            depth: 0x0,
            inverse: 0.0,
            point: [0x0, 0x0, 0x0],
            view: [0x0, 0x0, 0x0],
            matrix_copy: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            turned: [0.0, 0.0, 0.0],
        }
    }
}

/// The snow curve's walk variables and records.
pub struct SnowCurveVars {
    /// The curve walk's value, least step, most step, change, step and the running sum of the seeds
    /// (`effects::make_noise_curve`).
    pub curve_vars: [u32; 6],
    /// 0, then the flake count (`effects::draw_snow`; nothing reads it).
    pub first: u32,
    /// 0x20 records of 0x80 bytes (four quarters filled by `effects::make_noise_curve`; nothing
    /// reads them).
    pub records: [[u32; 32]; 32],
}

impl SnowCurveVars {
    pub const fn new() -> Self {
        SnowCurveVars {
            curve_vars: [0; 6],
            first: 0,
            records: [[0; 32]; 32],
        }
    }
}

/// The snow's light, fade colours and constants.
pub struct SnowLight {
    /// The light's place turned by the view (`effects::run_snow`).
    pub light_place: [f32; 3],
    /// Its way turned.
    pub light_way: [f32; 3],
    /// The flake's place less the light's (`effects::draw_snow`).
    pub fade_delta: [f32; 3],
    /// That made of length 1, times the way.
    pub fade_unit: [f32; 3],
    /// The fade colour chosen.
    pub fade_index: u32,
    /// 17 colours from the view's light toward white (`effects::make_fade_colours`).
    pub fade_colours: [u32; 17],
    /// The fade's step, growing from 0 (a float).
    pub fade_step: f32,
    /// The light colour's components' inverses.
    pub fade_inverses: [u32; 3],
    /// The inverses times the step.
    pub fade_scaled: [u32; 3],
    /// The settings' `snow_light_words[1]`.
    pub kept: u32,
    /// The light's dot with the flake's way, squared (`effects::draw_snow`).
    pub dot: f32,
    /// 8.5, what the curve's walk value (shifted left 4) is multiplied by.
    pub curve_scale: f32,
    /// 0.9, the walk's damping.
    pub curve_damp: f32,
    /// 0.3, 0.4, 0.3, the smoothing's weights.
    pub smooth_weights: [f32; 3],
    /// 0x1000, the ramp's length.
    pub ramp_by: i32,
    /// 0.0625.
    pub spread_scale: f32,
    /// Added to a spread.
    pub spread_base: f32,
    /// 0xC00 each, what the remainders are taken by.
    pub divisors: [u32; 2],
    /// The values the curve records' quarters are filled with.
    pub record_values: [f32; 5],
    /// Loaded into the first quad.
    pub quad_first: f32,
    /// The light's place before it is turned.
    pub light_source: [f32; 3],
    /// Its way before it is turned.
    pub light_source_way: [f32; 3],
    /// The vertices' rhw.
    pub rhw: f32,
    /// The quad's texture places (u0, v0, u1, v1).
    pub uv: [f32; 4],
    /// What the clock's step is multiplied by for the fall.
    pub fall_k: f32,
    /// The view's height (`effects::place_snow_view`).
    pub height: f32,
    /// A flake's side over its depth.
    pub side: f32,
    /// Added to a flake's u.
    pub u_offset: f32,
    /// The top and bottom v of a flake.
    pub v: [f32; 2],
}

impl SnowLight {
    pub const fn new() -> Self {
        SnowLight {
            light_place: [0.0, 0.0, 0.0],
            light_way: [0.0, 0.0, 0.0],
            fade_delta: [0.0, 0.0, 0.0],
            fade_unit: [0.0, 0.0, 0.0],
            fade_index: 0x0,
            fade_colours: [0; 17],
            fade_step: 0.0,
            fade_inverses: [0x0, 0x0, 0x0],
            fade_scaled: [0x0, 0x0, 0x0],
            kept: 0x0,
            dot: 0.0,
            curve_scale: 8.5,
            curve_damp: 0.9,
            smooth_weights: [0.3, 0.4, 0.3],
            ramp_by: 0x1000,
            spread_scale: 0.0625,
            spread_base: 0.001953125,
            divisors: [0xC00, 0xC00],
            record_values: [256.0, 0.001953125, 0.001953125, 0.064453125, 0.064453125],
            quad_first: 6200.0,
            light_source: [131072.0, 0.0, 0.0],
            light_source_way: [1.0, 0.0, 0.0],
            rhw: 256.0,
            uv: [0.001953125, 0.001953125, 0.064453125, 0.064453125],
            fall_k: -160.0,
            height: 524288.0,
            side: 6200.0,
            u_offset: 0.0625,
            v: [0.001953125, 0.064453125],
        }
    }
}
