//! The snow's tables: `SnowTables` (IDA
//! 0x8C6000 .. 0x8D2000: the flakes' randoms, remainders and spreads) and `SnowCurve`
//! (0x8D2310 .. 0x8DA310: the curve and its copy), `SnowState` (0x8D2000 .. 0x8D2310: the
//! flakes' variables, with the 16-byte constants), `SnowCurveVars` (0x8DA410 .. 0x8DB430:
//! the walk's variables and the curve records) and `SnowLight` (0x8DB430 .. 0x8DB558: the
//! light, the fade colours and the constants).

use crate::session::{PlainDword, PlainFloat};

/// The snow's random tables (0x800 flakes) (IDA 0x8C6000 .. 0x8D2000).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowTables {
    /// 0x8C6000: `g_marksRandomsA` (0x8C6000), `g_marksRandomsB` (0x8C8000), `g_marksRandomsC` (0x8CA000): the flakes' places, 20-bit randoms (`effects::make_noise_curve`).
    pub randoms: [[PlainDword; 2048]; 3],
    /// 0x8CC000: `g_marksRemaindersA` (0x8CC000), `g_marksRemaindersB` (0x8CE000): remainders of randoms (the curve indices' offsets for x and z).
    pub remainders: [[PlainDword; 2048]; 2],
    /// 0x8D0000: `g_marksSpreads`: the flakes' texture u (three bits of the seed times 0.0625 plus `flt_8DB4E8`).
    pub spreads: [PlainFloat; 2048],
}

impl SnowTables {
    pub const fn new() -> Self {
        SnowTables {
            randoms: [const { [const { PlainDword::new(0x0) }; 2048] }; 3],
            remainders: [const { [const { PlainDword::new(0x0) }; 2048] }; 2],
            spreads: [const { PlainFloat::new(0.0) }; 2048],
        }
    }
}

/// The snow's curve (IDA 0x8D2310 .. 0x8DA310).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowCurve {
    /// 0x8D2310: `g_marksCurve`: 0x1000 whole numbers, a smoothed random walk the flakes' x and z are moved by (`effects::make_noise_curve`).
    pub curve: [PlainDword; 4096],
    /// 0x8D6310: `g_marksCurveCopy`: the curve copied while it is smoothed, then the curve less a ramp.
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

/// The snow's variables (IDA 0x8D2000 .. 0x8D2310).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowState {
    /// 0x8D2030: `xmmword_8D2030` .. `xmmword_8D20B0`: the turn's nine floats, each four times (`effects::run_snow`).
    pub wide_turn: [[PlainFloat; 4]; 9],
    /// 0x8D20C0: `dword_8D20C0` .. `dword_8D20DC`: cleared by `effects::place_snow_view`.
    pub cleared: [PlainDword; 8],
    /// 0x8D20E0: `xmmword_8D20E0`: the view point's z, four times (`effects::place_snow_view`).
    pub filled: [PlainFloat; 4],
    /// 0x8D20F0: `xmmword_8D20F0`: the focal length (`flt_624088`) four times.
    pub wide_focal: [PlainFloat; 4],
    /// 0x8D2100: `xmmword_8D2100`: `flt_8DB508` loaded, four times (`effects::make_noise_curve`).
    pub quad_a: [PlainFloat; 4],
    /// 0x8D2110: `flt_8D2110` .. `flt_8D211C`: one over `flt_6241C8`, four times (and `inverse` again).
    pub inverses: [PlainFloat; 4],
    /// 0x8D2120: `xmmword_8D2120`: `flt_62409C` four times.
    pub quad_b: [PlainFloat; 4],
    /// 0x8D2130: `xmmword_8D2130`: `flt_6240A0` four times.
    pub quad_c: [PlainFloat; 4],
    /// 0x8D2140: `xmmword_8D2140`: -1.0 four times (nothing ported reads it).
    pub minus_ones: [PlainFloat; 4],
    /// 0x8D2150: `dword_8D2150`: the scene clock at the last snow frame.
    pub last_clock: PlainDword,
    /// 0x8D215C: `dword_8D215C`: 0 with no flakes (and after `effects::make_noise_curve`).
    pub flag: PlainDword,
    /// 0x8D2160: `dword_8D2160` ..: the settings' +0x1C .. +0x24, the view's place.
    pub view_place: [PlainDword; 3],
    /// 0x8D216C: `dword_8D216C` ..: the settings' +0x28 .. +0x48, the view's 3 x 3 matrix.
    pub view_matrix: [PlainFloat; 9],
    /// 0x8D2194: `dword_8D2194`: how far the flakes have fallen (28 bits).
    pub fall: PlainDword,
    /// 0x8D219C: `dword_8D219C`: the flakes drawn this frame.
    pub drawn: PlainDword,
    /// 0x8D21A0: `dword_8D21A0`: a copy of `flakes`.
    pub count_copy: PlainDword,
    /// 0x8D21A4: `dword_8D21A4`: the flakes (the settings' +0x2760, held to 0x800; initially 0x800).
    pub flakes: PlainDword,
    /// 0x8D21F4: `dword_8D21F4`: the flake's depth as a whole number.
    pub depth: PlainDword,
    /// 0x8D21F8: `flt_8D21F8`: one over `flt_6241C8`.
    pub inverse: PlainFloat,
    /// 0x8D2208: `dword_8D2208` ..: the point `effects::place_snow_view` turns (whole numbers).
    pub point: [PlainDword; 3],
    /// 0x8D2214: `dword_8D2214` ..: the view's place less the point (`effects::place_snow_view`).
    pub view: [PlainDword; 3],
    /// 0x8D2220: `dword_8D2220` ..: the view matrix copied (`effects::place_snow_view`).
    pub matrix_copy: [PlainFloat; 9],
    /// 0x8D22D4: `flt_8D22D4` ..: the flake's place turned.
    pub turned: [PlainFloat; 3],
}

impl SnowState {
    pub const fn new() -> Self {
        SnowState {
            wide_turn: [const {
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ]
            }; 9],
            cleared: [const { PlainDword::new(0x0) }; 8],
            filled: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            wide_focal: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            quad_a: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            inverses: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            quad_b: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            quad_c: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            minus_ones: [
                PlainFloat::new(-1.0),
                PlainFloat::new(-1.0),
                PlainFloat::new(-1.0),
                PlainFloat::new(-1.0),
            ],
            last_clock: PlainDword::new(0x0),
            flag: PlainDword::new(0x0),
            view_place: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            view_matrix: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            fall: PlainDword::new(0x0),
            drawn: PlainDword::new(0x0),
            count_copy: PlainDword::new(0x0),
            flakes: PlainDword::new(0x800),
            depth: PlainDword::new(0x0),
            inverse: PlainFloat::new(0.0),
            point: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            view: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            matrix_copy: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            turned: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}

/// The snow curve's walk variables and records (IDA 0x8DA410 .. 0x8DB430).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowCurveVars {
    /// 0x8DA410: `dword_8DA410` ..: the curve walk's value, least step, most step, change, step and the running sum of the seeds (`effects::make_noise_curve`).
    pub curve_vars: [PlainDword; 6],
    /// 0x8DA428: `dword_8DA428`: 0, then the flake count (`effects::draw_snow`; nothing reads it).
    pub first: PlainDword,
    /// 0x8DA430: `g_marksCurveRecords`: 0x20 records of 0x80 bytes (four quarters filled by `effects::make_noise_curve`; nothing ported reads them).
    pub records: [[PlainDword; 32]; 32],
}

impl SnowCurveVars {
    pub const fn new() -> Self {
        SnowCurveVars {
            curve_vars: [const { PlainDword::new(0x0) }; 6],
            first: PlainDword::new(0x0),
            records: [const { [const { PlainDword::new(0x0) }; 32] }; 32],
        }
    }
}

/// The snow's light, fade colours and constants (IDA 0x8DB430 .. 0x8DB558).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowLight {
    /// 0x8DB430: `flt_8DB430` ..: the light's place turned by the view (`effects::run_snow`).
    pub light_place: [PlainFloat; 3],
    /// 0x8DB43C: `flt_8DB43C` ..: its way turned.
    pub light_way: [PlainFloat; 3],
    /// 0x8DB448: `flt_8DB448` ..: the flake's place less the light's (`effects::draw_snow`).
    pub fade_delta: [PlainFloat; 3],
    /// 0x8DB454: `flt_8DB454` ..: that made of length 1, times the way.
    pub fade_unit: [PlainFloat; 3],
    /// 0x8DB460: `dword_8DB460`: the fade colour chosen.
    pub fade_index: PlainDword,
    /// 0x8DB464: `g_marksFadeColours`: 17 colours from the view's light toward white (`effects::make_fade_colours`).
    pub fade_colours: [PlainDword; 17],
    /// 0x8DB4A8: `dword_8DB4A8`: the fade's step, growing from 0 (a float).
    pub fade_step: PlainFloat,
    /// 0x8DB4AC: `dword_8DB4AC` ..: the light colour's components' inverses.
    pub fade_inverses: [PlainDword; 3],
    /// 0x8DB4B8: `dword_8DB4B8` ..: the inverses times the step.
    pub fade_scaled: [PlainDword; 3],
    /// 0x8DB4C4: `dword_8DB4C4`: the settings' +0x58.
    pub kept: PlainDword,
    /// 0x8DB4C8: `flt_8DB4C8`: the light's dot with the flake's way, squared (`effects::draw_snow`).
    pub dot: PlainFloat,
    /// 0x8DB4CC: `flt_8DB4CC`: 8.5, what the curve's walk value (shifted left 4) is multiplied by.
    pub curve_scale: PlainFloat,
    /// 0x8DB4D0: `flt_8DB4D0`: 0.9, the walk's damping.
    pub curve_damp: PlainFloat,
    /// 0x8DB4D4: `flt_8DB4D4`, `flt_8DB4D8`, `flt_8DB4DC`: 0.3, 0.4, 0.3, the smoothing's weights.
    pub smooth_weights: [PlainFloat; 3],
    /// 0x8DB4E0: `dword_8DB4E0`: 0x1000, the ramp's length.
    pub ramp_by: PlainDword,
    /// 0x8DB4E4: `flt_8DB4E4`: 0.0625.
    pub spread_scale: PlainFloat,
    /// 0x8DB4E8: `flt_8DB4E8`: added to a spread.
    pub spread_base: PlainFloat,
    /// 0x8DB4EC: `dword_8DB4EC`, `dword_8DB4F0`: 0xC00 each, what the remainders are taken by.
    pub divisors: [PlainDword; 2],
    /// 0x8DB4F4: `dword_8DB4F4` .. `dword_8DB504`: the values the curve records' quarters are filled with.
    pub record_values: [PlainFloat; 5],
    /// 0x8DB508: `flt_8DB508`: loaded into the first quad.
    pub quad_first: PlainFloat,
    /// 0x8DB50C: `flt_8DB50C` ..: the light's place before it is turned.
    pub light_source: [PlainFloat; 3],
    /// 0x8DB518: `flt_8DB518` ..: its way before it is turned.
    pub light_source_way: [PlainFloat; 3],
    /// 0x8DB524: `dword_8DB524`: the vertices' rhw.
    pub rhw: PlainFloat,
    /// 0x8DB528: `dword_8DB528` ..: the quad's texture places (u0, v0, u1, v1).
    pub uv: [PlainFloat; 4],
    /// 0x8DB538: `flt_8DB538`: what the clock's step is multiplied by for the fall.
    pub fall_k: PlainFloat,
    /// 0x8DB53C: `flt_8DB53C`: the view's height (`effects::place_snow_view`).
    pub height: PlainFloat,
    /// 0x8DB548: `flt_8DB548`: a flake's side over its depth.
    pub side: PlainFloat,
    /// 0x8DB54C: `flt_8DB54C`: added to a flake's u.
    pub u_offset: PlainFloat,
    /// 0x8DB550: `dword_8DB550`, `dword_8DB554`: the top and bottom v of a flake.
    pub v: [PlainFloat; 2],
}

impl SnowLight {
    pub const fn new() -> Self {
        SnowLight {
            light_place: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            light_way: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            fade_delta: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            fade_unit: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            fade_index: PlainDword::new(0x0),
            fade_colours: [const { PlainDword::new(0x0) }; 17],
            fade_step: PlainFloat::new(0.0),
            fade_inverses: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            fade_scaled: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            kept: PlainDword::new(0x0),
            dot: PlainFloat::new(0.0),
            curve_scale: PlainFloat::new(8.5),
            curve_damp: PlainFloat::new(0.9),
            smooth_weights: [
                PlainFloat::new(0.3),
                PlainFloat::new(0.4),
                PlainFloat::new(0.3),
            ],
            ramp_by: PlainDword::new(0x1000),
            spread_scale: PlainFloat::new(0.0625),
            spread_base: PlainFloat::new(0.001953125),
            divisors: [PlainDword::new(0xC00), PlainDword::new(0xC00)],
            record_values: [
                PlainFloat::new(256.0),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.064453125),
                PlainFloat::new(0.064453125),
            ],
            quad_first: PlainFloat::new(6200.0),
            light_source: [
                PlainFloat::new(131072.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            light_source_way: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            rhw: PlainFloat::new(256.0),
            uv: [
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.064453125),
                PlainFloat::new(0.064453125),
            ],
            fall_k: PlainFloat::new(-160.0),
            height: PlainFloat::new(524288.0),
            side: PlainFloat::new(6200.0),
            u_offset: PlainFloat::new(0.0625),
            v: [PlainFloat::new(0.001953125), PlainFloat::new(0.064453125)],
        }
    }
}
