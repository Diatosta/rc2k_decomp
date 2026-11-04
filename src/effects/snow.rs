//! The snow's tables, out of the marks' block (phase 7, typed state): `SnowTables` (IDA
//! 0x8C6000 .. 0x8D2000: the flakes' randoms, remainders and spreads) and `SnowCurve`
//! (0x8D2310 .. 0x8DA310: the curve and its copy), `SnowState` (0x8D2000 .. 0x8D2310: the
//! flakes' variables, with the 16-byte constants), `SnowCurveVars` (0x8DA410 .. 0x8DB430:
//! the walk's variables and the curve records) and `SnowLight` (0x8DB430 .. 0x8DB558: the
//! light, the fade colours and the constants).

use crate::session::{Bytes, Dword, Float};

/// The snow's random tables (0x800 flakes) (IDA 0x8C6000 .. 0x8D2000, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowTables {
    /// 0x8C6000: `g_marksRandomsA` (0x8C6000), `g_marksRandomsB` (0x8C8000), `g_marksRandomsC` (0x8CA000): the flakes' places, 20-bit randoms (`effects::make_noise_curve`).
    pub randoms: [[Dword; 2048]; 3],
    /// 0x8CC000: `g_marksRemaindersA` (0x8CC000), `g_marksRemaindersB` (0x8CE000): remainders of randoms (the curve indices' offsets for x and z).
    pub remainders: [[Dword; 2048]; 2],
    /// 0x8D0000: `g_marksSpreads`: the flakes' texture u (three bits of the seed times 0.0625 plus `flt_8DB4E8`).
    pub spreads: [Float; 2048],
}

pub static SNOW_TABLES: SnowTables = SnowTables {
    randoms: [const { [const { Dword::new(0x0) }; 2048] }; 3],
    remainders: [const { [const { Dword::new(0x0) }; 2048] }; 2],
    spreads: [const { Float::from_bits(0x0) }; 2048],
};

impl SnowTables {
    pub const START: usize = 0x8C6000;
    pub const LEN: usize = 0xC000;
}

/// The snow's curve (IDA 0x8D2310 .. 0x8DA310, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowCurve {
    /// 0x8D2310: `g_marksCurve`: 0x1000 whole numbers, a smoothed random walk the flakes' x and z are moved by (`effects::make_noise_curve`).
    pub curve: [Dword; 4096],
    /// 0x8D6310: `g_marksCurveCopy`: the curve copied while it is smoothed, then the curve less a ramp.
    pub copy: [Dword; 4096],
}

pub static SNOW_CURVE: SnowCurve = SnowCurve {
    curve: [const { Dword::new(0x0) }; 4096],
    copy: [const { Dword::new(0x0) }; 4096],
};

impl SnowCurve {
    pub const START: usize = 0x8D2310;
    pub const LEN: usize = 0x8000;
}

/// The snow's variables (IDA 0x8D2000 .. 0x8D2310, the image's layout; phase 7, typed state).
#[repr(C, align(16))]
pub struct SnowState {
    /// 0x8D2000: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_8d2000: Bytes<48>,
    /// 0x8D2030: `xmmword_8D2030` .. `xmmword_8D20B0`: the turn's nine floats, each four times (`effects::run_snow`).
    pub wide_turn: [[Float; 4]; 9],
    /// 0x8D20C0: `dword_8D20C0` .. `dword_8D20DC`: cleared by `effects::place_view_2`.
    pub cleared: [Dword; 8],
    /// 0x8D20E0: `xmmword_8D20E0`: the view point's z, four times (`effects::place_view_2`).
    pub filled: [Float; 4],
    /// 0x8D20F0: `xmmword_8D20F0`: the focal length (`flt_624088`) four times.
    pub wide_focal: [Float; 4],
    /// 0x8D2100: `xmmword_8D2100`: `flt_8DB508` loaded, four times (`effects::make_noise_curve`).
    pub quad_a: [Float; 4],
    /// 0x8D2110: `flt_8D2110` .. `flt_8D211C`: one over `flt_6241C8`, four times (and `inverse` again).
    pub inverses: [Float; 4],
    /// 0x8D2120: `xmmword_8D2120`: `flt_62409C` four times.
    pub quad_b: [Float; 4],
    /// 0x8D2130: `xmmword_8D2130`: `flt_6240A0` four times.
    pub quad_c: [Float; 4],
    /// 0x8D2140: `xmmword_8D2140`: -1.0 four times (nothing ported reads it).
    pub minus_ones: [Float; 4],
    /// 0x8D2150: `dword_8D2150`: the scene clock at the last snow frame.
    pub last_clock: Dword,
    /// 0x8D2154: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_8d2154: Bytes<8>,
    /// 0x8D215C: `dword_8D215C`: 0 with no flakes (and after `effects::make_noise_curve`).
    pub flag: Dword,
    /// 0x8D2160: `dword_8D2160` ..: the settings' +0x1C .. +0x24, the view's place.
    pub view_place: [Dword; 3],
    /// 0x8D216C: `dword_8D216C` ..: the settings' +0x28 .. +0x48, the view's 3 x 3 matrix.
    pub view_matrix: [Float; 9],
    /// 0x8D2190: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8d2190: Bytes<4>,
    /// 0x8D2194: `dword_8D2194`: how far the flakes have fallen (28 bits).
    pub fall: Dword,
    /// 0x8D2198: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8d2198: Bytes<4>,
    /// 0x8D219C: `dword_8D219C`: the flakes drawn this frame.
    pub drawn: Dword,
    /// 0x8D21A0: `dword_8D21A0`: a copy of `flakes`.
    pub count_copy: Dword,
    /// 0x8D21A4: `dword_8D21A4`: the flakes (the settings' +0x2760, held to 0x800; initially 0x800).
    pub flakes: Dword,
    /// 0x8D21A8: Not named: 76 bytes (a gap, or unreferenced data).
    pub _gap_8d21a8: Bytes<76>,
    /// 0x8D21F4: `dword_8D21F4`: the flake's depth as a whole number.
    pub depth: Dword,
    /// 0x8D21F8: `flt_8D21F8`: one over `flt_6241C8`.
    pub inverse: Float,
    /// 0x8D21FC: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_8d21fc: Bytes<12>,
    /// 0x8D2208: `dword_8D2208` ..: the point `effects::place_view_2` turns (whole numbers).
    pub point: [Dword; 3],
    /// 0x8D2214: `dword_8D2214` ..: the view's place less the point (`effects::place_view_2`).
    pub view: [Dword; 3],
    /// 0x8D2220: `dword_8D2220` ..: the view matrix copied (`effects::place_view_2`).
    pub matrix_copy: [Float; 9],
    /// 0x8D2244: Not named: 144 bytes (a gap, or unreferenced data).
    pub _gap_8d2244: Bytes<144>,
    /// 0x8D22D4: `flt_8D22D4` ..: the flake's place turned.
    pub turned: [Float; 3],
    /// 0x8D22E0: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_8d22e0: Bytes<48>,
}

pub static SNOW: SnowState = SnowState {
    _gap_8d2000: Bytes::new([0; 48]),
    wide_turn: [const {
        [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ]
    }; 9],
    cleared: [const { Dword::new(0x0) }; 8],
    filled: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    wide_focal: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    quad_a: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    inverses: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    quad_b: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    quad_c: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    minus_ones: [
        Float::from_bits(0xBF800000),
        Float::from_bits(0xBF800000),
        Float::from_bits(0xBF800000),
        Float::from_bits(0xBF800000),
    ],
    last_clock: Dword::new(0x0),
    _gap_8d2154: Bytes::new([0; 8]),
    flag: Dword::new(0x0),
    view_place: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    view_matrix: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    _gap_8d2190: Bytes::new([0; 4]),
    fall: Dword::new(0x0),
    _gap_8d2198: Bytes::new([0; 4]),
    drawn: Dword::new(0x0),
    count_copy: Dword::new(0x0),
    flakes: Dword::new(0x800),
    _gap_8d21a8: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]),
    depth: Dword::new(0x0),
    inverse: Float::from_bits(0x0),
    _gap_8d21fc: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F,
    ]),
    point: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    view: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    matrix_copy: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    _gap_8d2244: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80,
        0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x80, 0x3F, 0x00, 0x80, 0x31, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x31, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
    turned: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    _gap_8d22e0: Bytes::new([0; 48]),
};

impl SnowState {
    pub const START: usize = 0x8D2000;
    pub const LEN: usize = 0x310;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SNOW) as usize
    }
}

/// The snow curve's walk variables and records (IDA 0x8DA410 .. 0x8DB430, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowCurveVars {
    /// 0x8DA410: `dword_8DA410` ..: the curve walk's value, least step, most step, change, step and the running sum of the seeds (`effects::make_noise_curve`).
    pub curve_vars: [Dword; 6],
    /// 0x8DA428: `dword_8DA428`: 0, then the flake count (`effects::draw_snow`; nothing reads it).
    pub first: Dword,
    /// 0x8DA42C: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8da42c: Bytes<4>,
    /// 0x8DA430: `g_marksCurveRecords`: 0x20 records of 0x80 bytes (four quarters filled by `effects::make_noise_curve`; nothing ported reads them).
    pub records: [[Dword; 32]; 32],
}

pub static SNOW_VARS: SnowCurveVars = SnowCurveVars {
    curve_vars: [const { Dword::new(0x0) }; 6],
    first: Dword::new(0x0),
    _gap_8da42c: Bytes::new([0; 4]),
    records: [const { [const { Dword::new(0x0) }; 32] }; 32],
};

impl SnowCurveVars {
    pub const START: usize = 0x8DA410;
    pub const LEN: usize = 0x1020;
}

/// The snow's light, fade colours and constants (IDA 0x8DB430 .. 0x8DB558, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SnowLight {
    /// 0x8DB430: `flt_8DB430` ..: the light's place turned by the view (`effects::run_snow`).
    pub light_place: [Float; 3],
    /// 0x8DB43C: `flt_8DB43C` ..: its way turned.
    pub light_way: [Float; 3],
    /// 0x8DB448: `flt_8DB448` ..: the flake's place less the light's (`effects::draw_snow`).
    pub fade_delta: [Float; 3],
    /// 0x8DB454: `flt_8DB454` ..: that made of length 1, times the way.
    pub fade_unit: [Float; 3],
    /// 0x8DB460: `dword_8DB460`: the fade colour chosen.
    pub fade_index: Dword,
    /// 0x8DB464: `g_marksFadeColours`: 17 colours from the view's light toward white (`effects::make_fade_colours`).
    pub fade_colours: [Dword; 17],
    /// 0x8DB4A8: `dword_8DB4A8`: the fade's step, growing from 0 (a float).
    pub fade_step: Float,
    /// 0x8DB4AC: `dword_8DB4AC` ..: the light colour's components' inverses.
    pub fade_inverses: [Dword; 3],
    /// 0x8DB4B8: `dword_8DB4B8` ..: the inverses times the step.
    pub fade_scaled: [Dword; 3],
    /// 0x8DB4C4: `dword_8DB4C4`: the settings' +0x58.
    pub kept: Dword,
    /// 0x8DB4C8: `flt_8DB4C8`: the light's dot with the flake's way, squared (`effects::draw_snow`).
    pub dot: Float,
    /// 0x8DB4CC: `flt_8DB4CC`: 8.5, what the curve's walk value (shifted left 4) is multiplied by.
    pub curve_scale: Float,
    /// 0x8DB4D0: `flt_8DB4D0`: 0.9, the walk's damping.
    pub curve_damp: Float,
    /// 0x8DB4D4: `flt_8DB4D4`, `flt_8DB4D8`, `flt_8DB4DC`: 0.3, 0.4, 0.3, the smoothing's weights.
    pub smooth_weights: [Float; 3],
    /// 0x8DB4E0: `dword_8DB4E0`: 0x1000, the ramp's length.
    pub ramp_by: Dword,
    /// 0x8DB4E4: `flt_8DB4E4`: 0.0625.
    pub spread_scale: Float,
    /// 0x8DB4E8: `flt_8DB4E8`: added to a spread.
    pub spread_base: Float,
    /// 0x8DB4EC: `dword_8DB4EC`, `dword_8DB4F0`: 0xC00 each, what the remainders are taken by.
    pub divisors: [Dword; 2],
    /// 0x8DB4F4: `dword_8DB4F4` .. `dword_8DB504`: the values the curve records' quarters are filled with.
    pub record_values: [Float; 5],
    /// 0x8DB508: `flt_8DB508`: loaded into the first quad.
    pub quad_first: Float,
    /// 0x8DB50C: `flt_8DB50C` ..: the light's place before it is turned.
    pub light_source: [Float; 3],
    /// 0x8DB518: `flt_8DB518` ..: its way before it is turned.
    pub light_source_way: [Float; 3],
    /// 0x8DB524: `dword_8DB524`: the vertices' rhw.
    pub rhw: Dword,
    /// 0x8DB528: `dword_8DB528` ..: the quad's texture places (u0, v0, u1, v1).
    pub uv: [Dword; 4],
    /// 0x8DB538: `flt_8DB538`: what the clock's step is multiplied by for the fall.
    pub fall_k: Float,
    /// 0x8DB53C: `flt_8DB53C`: the view's height (`effects::place_view_2`).
    pub height: Float,
    /// 0x8DB540: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_8db540: Bytes<8>,
    /// 0x8DB548: `flt_8DB548`: a flake's side over its depth.
    pub side: Float,
    /// 0x8DB54C: `flt_8DB54C`: added to a flake's u.
    pub u_offset: Float,
    /// 0x8DB550: `dword_8DB550`, `dword_8DB554`: the top and bottom v of a flake.
    pub v: [Dword; 2],
}

pub static SNOW_LIGHT: SnowLight = SnowLight {
    light_place: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    light_way: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    fade_delta: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    fade_unit: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    fade_index: Dword::new(0x0),
    fade_colours: [const { Dword::new(0x0) }; 17],
    fade_step: Float::from_bits(0x0),
    fade_inverses: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    fade_scaled: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    kept: Dword::new(0x0),
    dot: Float::from_bits(0x0),
    curve_scale: Float::from_bits(0x41080000),
    curve_damp: Float::from_bits(0x3F666666),
    smooth_weights: [
        Float::from_bits(0x3E99999A),
        Float::from_bits(0x3ECCCCCD),
        Float::from_bits(0x3E99999A),
    ],
    ramp_by: Dword::new(0x1000),
    spread_scale: Float::from_bits(0x3D800000),
    spread_base: Float::from_bits(0x3B000000),
    divisors: [Dword::new(0xC00), Dword::new(0xC00)],
    record_values: [
        Float::from_bits(0x43800000),
        Float::from_bits(0x3B000000),
        Float::from_bits(0x3B000000),
        Float::from_bits(0x3D840000),
        Float::from_bits(0x3D840000),
    ],
    quad_first: Float::from_bits(0x45C1C000),
    light_source: [
        Float::from_bits(0x48000000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    light_source_way: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    rhw: Dword::new(0x43800000),
    uv: [
        Dword::new(0x3B000000),
        Dword::new(0x3B000000),
        Dword::new(0x3D840000),
        Dword::new(0x3D840000),
    ],
    fall_k: Float::from_bits(0xC3200000),
    height: Float::from_bits(0x49000000),
    _gap_8db540: Bytes::new([0x00, 0xC0, 0xC1, 0x45, 0x00, 0x00, 0x00, 0x00]),
    side: Float::from_bits(0x45C1C000),
    u_offset: Float::from_bits(0x3D800000),
    v: [Dword::new(0x3B000000), Dword::new(0x3D840000)],
};

impl SnowLight {
    pub const START: usize = 0x8DB430;
    pub const LEN: usize = 0x128;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same<T>(v: &T, start: usize, len: usize) -> bool {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        let mine: Vec<u8> = (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        mine == crate::moved::image_bytes(start, len)
    }

    #[test]
    fn the_tables_start_as_the_games() {
        assert_eq!(size_of::<SnowTables>(), SnowTables::LEN);
        assert_eq!(size_of::<SnowCurve>(), SnowCurve::LEN);
        assert!(
            same(&SNOW_TABLES, SnowTables::START, SnowTables::LEN),
            "tables"
        );
        assert!(same(&SNOW_CURVE, SnowCurve::START, SnowCurve::LEN), "curve");
        assert_eq!(size_of::<SnowState>(), SnowState::LEN);
        assert_eq!(size_of::<SnowCurveVars>(), SnowCurveVars::LEN);
        assert_eq!(size_of::<SnowLight>(), SnowLight::LEN);
        assert!(same(&SNOW, SnowState::START, SnowState::LEN), "state");
        assert!(
            same(&SNOW_VARS, SnowCurveVars::START, SnowCurveVars::LEN),
            "vars"
        );
        assert!(same(&SNOW_LIGHT, SnowLight::START, SnowLight::LEN), "light");
        assert_eq!(SNOW.wide_turn[0][0].addr() % 16, 0);
        assert_eq!(
            offset_of!(SnowState, last_clock),
            0x8D2150 - SnowState::START
        );
        assert_eq!(offset_of!(SnowState, fall), 0x8D2194 - SnowState::START);
        assert_eq!(offset_of!(SnowState, flakes), 0x8D21A4 - SnowState::START);
        assert_eq!(offset_of!(SnowState, turned), 0x8D22D4 - SnowState::START);
        assert_eq!(
            offset_of!(SnowCurveVars, records),
            0x8DA430 - SnowCurveVars::START
        );
        assert_eq!(
            offset_of!(SnowLight, fade_colours),
            0x8DB464 - SnowLight::START
        );
        assert_eq!(
            offset_of!(SnowLight, quad_first),
            0x8DB508 - SnowLight::START
        );
        assert_eq!(offset_of!(SnowLight, v), 0x8DB550 - SnowLight::START);
        use core::mem::offset_of;
        assert_eq!(
            offset_of!(SnowTables, remainders),
            0x8CC000 - SnowTables::START
        );
        assert_eq!(
            offset_of!(SnowTables, spreads),
            0x8D0000 - SnowTables::START
        );
        assert_eq!(offset_of!(SnowCurve, copy), 0x8D6310 - SnowCurve::START);
    }
}
