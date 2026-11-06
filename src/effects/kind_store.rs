//! The marks' kind work, out of the marks' block (phase 7, typed state), IDA 0x9331B4 ..
//! 0x934CC8: `kind_values` (`init_marks`' steps 8 and 9) works out, for the marks' kind
//! (`g_marksKind`), the values of two quads from two sets of tables, and `draw_shadow` and
//! `draw_plane` draw the three quads from them:
//!
//! - `QuadWork` (0xF8 bytes), three times: set 1 (`QUAD_1`, 0x9331B4: `draw_shadow`'s
//!   `SECOND`, the second shadow, drawn by `draw_overlays`), set 2 (`QUAD_2`, 0x933ED0:
//!   `draw_shadow`'s `FIRST`) and the plane's (`QUAD_3`, 0x934BD0: `draw_plane`, whose values
//!   a third set of tables would have made; nothing in the port writes but the corner and the
//!   corners);
//! - `KindTables` (the per-kind tables of 0x22 entries), `KindExtras1` / `KindExtra2` (the
//!   `extra` out and, for set 1, the `divisor` copy) and `KindConsts` (the constants and the
//!   light), once for each set. Set 1 is `kind_values::FIRST`, set 2 `kind_values::SECOND`.
//!
//! The code between them in the image (0x9332AC .. 0x933458 and 0x933FC8 .. 0x934160: a
//! string and unreferenced data) is the tables' leading gap.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{Bytes, Dword, Float};

/// A quad's work area, the image's layout (IDA 0xF8 bytes, three copies: `QUAD_1`, `QUAD_2`, `QUAD_3`; phase 7, typed state). The offsets are from its start.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct QuadWork {
    /// +0x0: the quad's place (z, y, x as `kind_values` writes it, a reading): the kind's two triples blended (set 1 `flt_9331B4`, `flt_9331B8` ..; set 2 `flt_933ED0`, `flt_933ED4` ..; plane `flt_934BD0`, `flt_934BD4` ..)
    pub place: [Float; 3],
    /// +0xC: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_c: Bytes<4>,
    /// +0x10: the light's way turned by the matrix (`draw_shadow`), a reading (set 1 `flt_9331C4`, `flt_9331C8` ..; set 2 `flt_933EE0`, `flt_933EE4` ..; plane `flt_934BE0`, `flt_934BE4` ..)
    pub light_turned: [Float; 3],
    /// +0x1C: the matrix turned about z by the angle (`turn_matrix3`); its rows are made the frame's (set 1 `dword_9331D0`, `dword_9331D4` ..; set 2 `dword_933EEC`, `dword_933EF0` ..; plane `dword_934BEC`, `dword_934BF0` ..)
    pub matrix: [Float; 9],
    /// +0x40: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_40: Bytes<12>,
    /// +0x4C: the quad's two sides: the kind's triple (`copied`), then (0, difference y, minus difference x) (set 1 `dword_933200`, `dword_933204` ..; set 2 `dword_933F1C`, `dword_933F20` ..; plane `dword_934C1C`, `dword_934C20` ..)
    pub sides: [[Float; 3]; 2],
    /// +0x64: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_64: Bytes<12>,
    /// +0x70: the sides made one long and square (`normalise_vector3`, `cross_product3`); the light turned is stored in the third (set 1 `dword_933224`, `dword_933228` ..; set 2 `dword_933F40`, `dword_933F44` ..; plane `dword_934C40`, `dword_934C44` ..)
    pub frame: [[Float; 3]; 3],
    /// +0x94: the corner of `g_marksCorners` (-1: none), the kind's `count` (set 1 `dword_933248`; set 2 `dword_933F64`; plane `dword_934C64`)
    pub corner: Dword,
    /// +0x98: the kind's size over its divisor (`kind_values`) (set 1 `flt_93324C`; set 2 `flt_933F68`; plane `flt_934C68`)
    pub ratio: Float,
    /// +0x9C: the kind's size times its constant (`kind_values`) (set 1 `flt_933250`; set 2 `flt_933F6C`; plane `flt_934C6C`)
    pub size: Float,
    /// +0xA0: the kind's start times its constant (set 1 `flt_933254`; set 2 `flt_933F70`; plane `flt_934C70`)
    pub start: Float,
    /// +0xA4: the angle line below the split: the factor (set 1 `flt_933258`; set 2 `flt_933F74`; plane `flt_934C74`)
    pub step: Float,
    /// +0xA8: the split of the angle's two lines: the kind's `low` (signed compare of the bits) (set 1 `dword_93325C`; set 2 `dword_933F78`; plane `flt_934C78`)
    pub low: Float,
    /// +0xAC: the kind's `high` (set 1 `dword_933260`; set 2 `dword_933F7C`; plane `flt_934C7C`)
    pub high: Float,
    /// +0xB0: the angle line above the split: its base (set 1 `flt_933264`; set 2 `flt_933F80`; plane `flt_934C80`)
    pub middle: Float,
    /// +0xB4: the angle line above the split: its factor (set 1 `flt_933268`; set 2 `flt_933F84`; plane `flt_934C84`)
    pub slope: Float,
    /// +0xB8: the factor the quad is drawn in by (`corners_in`) (set 1 `flt_93326C`; set 2 `flt_933F88`; plane `flt_934C88`)
    pub spread: Float,
    /// +0xBC: the way between two corners, made one long (`normalise_vector3`) (set 1 `flt_933270`, `flt_933274` ..; set 2 `flt_933F8C`, `flt_933F90` ..; plane `flt_934C8C`, `flt_934C90` ..)
    pub way: [Float; 3],
    /// +0xC8: the quad's four corners (z, y, x), turned to the view then projected (set 1 `flt_93327C`, `flt_933280` ..; set 2 `flt_933F98`, `flt_933F9C` ..; plane `flt_934C98`, `flt_934C9C` ..)
    pub corners: [[Float; 3]; 4],
}

impl QuadWork {
    /// The second shadow's (`draw_shadow::SECOND`) quad (IDA 0x9331B4 .. 0x9332AC); the first values are the image's.
    pub const fn new_1() -> Self {
        Self {
            place: [
                Float::from_bits(0x0),
                Float::from_bits(0x47F53000),
                Float::from_bits(0x0),
            ],
            _gap_c: Bytes::new([0; 4]),
            light_turned: [const { Float::from_bits(0x0) }; 3],
            matrix: [const { Float::from_bits(0x0) }; 9],
            _gap_40: Bytes::new([0; 12]),
            sides: [
                [
                    Float::from_bits(0x3F800000),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ],
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0xBF800000),
                    Float::from_bits(0xBF800000),
                ],
            ],
            _gap_64: Bytes::new([0; 12]),
            frame: [
                [
                    Float::from_bits(0x3F800000),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ],
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0xBF800000),
                    Float::from_bits(0xBF800000),
                ],
                [const { Float::from_bits(0x0) }; 3],
            ],
            corner: Dword::new(0x4),
            ratio: Float::from_bits(0x47000000),
            size: Float::from_bits(0x47800000),
            start: Float::from_bits(0x0),
            step: Float::from_bits(0x0),
            low: Float::from_bits(0x0),
            high: Float::from_bits(0x0),
            middle: Float::from_bits(0x0),
            slope: Float::from_bits(0x0),
            spread: Float::from_bits(0x0),
            way: [const { Float::from_bits(0x0) }; 3],
            corners: [const { [const { Float::from_bits(0x0) }; 3] }; 4],
        }
    }
}

impl QuadWork {
    /// The first shadow's (`draw_shadow::FIRST`) quad (IDA 0x933ED0 .. 0x933FC8); the first values are the image's.
    pub const fn new_2() -> Self {
        Self {
            place: [
                Float::from_bits(0x0),
                Float::from_bits(0x47F53000),
                Float::from_bits(0x0),
            ],
            _gap_c: Bytes::new([0; 4]),
            light_turned: [const { Float::from_bits(0x0) }; 3],
            matrix: [const { Float::from_bits(0x0) }; 9],
            _gap_40: Bytes::new([0; 12]),
            sides: [
                [
                    Float::from_bits(0x3F800000),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ],
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0xBF800000),
                    Float::from_bits(0xBF800000),
                ],
            ],
            _gap_64: Bytes::new([0; 12]),
            frame: [
                [
                    Float::from_bits(0x3F800000),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ],
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0xBF800000),
                    Float::from_bits(0xBF800000),
                ],
                [const { Float::from_bits(0x0) }; 3],
            ],
            corner: Dword::new(0x4),
            ratio: Float::from_bits(0x47000000),
            size: Float::from_bits(0x47800000),
            start: Float::from_bits(0x0),
            step: Float::from_bits(0x0),
            low: Float::from_bits(0x0),
            high: Float::from_bits(0x0),
            middle: Float::from_bits(0x0),
            slope: Float::from_bits(0x0),
            spread: Float::from_bits(0x0),
            way: [const { Float::from_bits(0x0) }; 3],
            corners: [const { [const { Float::from_bits(0x0) }; 3] }; 4],
        }
    }
}

impl QuadWork {
    /// The plane's (`draw_plane`) quad (IDA 0x934BD0 .. 0x934CC8); the first values are the image's.
    pub const fn new_3() -> Self {
        Self {
            place: [
                Float::from_bits(0x0),
                Float::from_bits(0x47F53000),
                Float::from_bits(0x0),
            ],
            _gap_c: Bytes::new([0; 4]),
            light_turned: [const { Float::from_bits(0x0) }; 3],
            matrix: [const { Float::from_bits(0x0) }; 9],
            _gap_40: Bytes::new([0; 12]),
            sides: [
                [
                    Float::from_bits(0x3F800000),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ],
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0xBF800000),
                    Float::from_bits(0xBF800000),
                ],
            ],
            _gap_64: Bytes::new([0; 12]),
            frame: [
                [
                    Float::from_bits(0x3F800000),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ],
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0xBF800000),
                    Float::from_bits(0xBF800000),
                ],
                [const { Float::from_bits(0x0) }; 3],
            ],
            corner: Dword::new(0x4),
            ratio: Float::from_bits(0x47000000),
            size: Float::from_bits(0x47800000),
            start: Float::from_bits(0x0),
            step: Float::from_bits(0x0),
            low: Float::from_bits(0x0),
            high: Float::from_bits(0x0),
            middle: Float::from_bits(0x0),
            slope: Float::from_bits(0x0),
            spread: Float::from_bits(0x0),
            way: [const { Float::from_bits(0x0) }; 3],
            corners: [const { [const { Float::from_bits(0x0) }; 3] }; 4],
        }
    }
}

/// `QUAD_1`'s place in the image (IDA 0x9331B4).
pub struct Quad1;

impl Quad1 {
    pub const START: usize = 0x9331B4;
    pub const LEN: usize = 0xF8;
}

/// `QUAD_2`'s place in the image (IDA 0x933ED0).
pub struct Quad2;

impl Quad2 {
    pub const START: usize = 0x933ED0;
    pub const LEN: usize = 0xF8;
}

/// `QUAD_3`'s place in the image (IDA 0x934BD0).
pub struct Quad3;

impl Quad3 {
    pub const START: usize = 0x934BD0;
    pub const LEN: usize = 0xF8;
}

/// The per-kind tables of one set (IDA after a leading gap of `GAP` bytes: 3 tables of 0x22 triples, then 10 of 0x22 values; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct KindTables<const GAP: usize> {
    /// +0x0: Not named: `GAP` bytes (the image's code between the quad and the tables: a string and unreferenced data).
    pub _gap_0: [u8; GAP],
    /// +0x1AC: `copied`: per kind, the triple copied into the quad's first side (`dword_933458`, `dword_93345C` ..)
    pub copied: [[f32; 3]; 34],
    /// +0x344: `from`: per kind, the triple the place and the second side start from (`flt_9335F0`, `flt_9335F4` ..)
    pub from: [[f32; 3]; 34],
    /// +0x4DC: `to`: per kind, the triple they end at (`flt_933788`, `flt_93378C` ..)
    pub to: [[f32; 3]; 34],
    /// +0x674: `extra`: per kind (times `extra_k` into the lift along the up) (`flt_933920`, `flt_933924` ..)
    pub extra: [f32; 34],
    /// +0x6FC: `size`: per kind (`flt_9339A8`, `flt_9339AC` ..)
    pub size: [f32; 34],
    /// +0x784: `divisor`: per kind (`flt_933A30`, `flt_933A34` ..)
    pub divisor: [f32; 34],
    /// +0x80C: `spread`: per kind (`flt_933AB8`, `flt_933ABC` ..)
    pub spread: [f32; 34],
    /// +0x894: `count`: per kind, the corner of `g_marksCorners` (-1: none) (`dword_933B40`, `dword_933B44` ..)
    pub count: [u32; 34],
    /// +0x91C: `start`: per kind (`flt_933BC8`, `flt_933BCC` ..)
    pub start: [f32; 34],
    /// +0x9A4: `middle`: per kind (`flt_933C50`, `flt_933C54` ..)
    pub middle: [f32; 34],
    /// +0xA2C: `end`: per kind (`flt_933CD8`, `flt_933CDC` ..)
    pub end: [f32; 34],
    /// +0xAB4: `high`: per kind (`dword_933D60`, `flt_933D64` ..)
    pub high: [f32; 34],
    /// +0xB3C: `low`: per kind (`dword_933DE8`, `flt_933DEC` ..)
    pub low: [f32; 34],
}

/// Set 1's tables (IDA 0x9332AC .. 0x933E70); the first values are the image's.
pub static TABLES_1: KindTables<0x1AC> = KindTables {
    _gap_0: [
        0x53, 0x70, 0x61, 0x6E, 0x6E, 0x65, 0x72, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x3F, 0x3F,
        0x3F, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA,
        0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA,
        0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA,
        0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA,
        0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA,
        0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA,
        0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
    ],
    copied: [const {
        [
            f32::from_bits(0x3F800000),
            f32::from_bits(0x0),
            f32::from_bits(0x0),
        ]
    }; 34],
    from: [
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC4040000),
            f32::from_bits(0x44580000),
            f32::from_bits(0xC3ED0000),
        ],
        [
            f32::from_bits(0xC417C000),
            f32::from_bits(0x44568000),
            f32::from_bits(0xC3D00000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x445F4000),
            f32::from_bits(0x43790000),
        ],
        [
            f32::from_bits(0xC4134000),
            f32::from_bits(0x444F4000),
            f32::from_bits(0xC39E8000),
        ],
        [
            f32::from_bits(0xC41E4000),
            f32::from_bits(0x44614000),
            f32::from_bits(0x43970000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC40BC000),
            f32::from_bits(0x445A8000),
            f32::from_bits(0xC3C80000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC4298000),
            f32::from_bits(0x446A4000),
            f32::from_bits(0xC3D20000),
        ],
        [
            f32::from_bits(0xC4174000),
            f32::from_bits(0x44664000),
            f32::from_bits(0xC3C68000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC41B0000),
            f32::from_bits(0x44474000),
            f32::from_bits(0xC3F40000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC4144000),
            f32::from_bits(0x444F0000),
            f32::from_bits(0x43550000),
        ],
        [
            f32::from_bits(0xC417C000),
            f32::from_bits(0x446DC000),
            f32::from_bits(0xC3F68000),
        ],
    ],
    to: [
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC3F98000),
            f32::from_bits(0x44468000),
            f32::from_bits(0xC3C80000),
        ],
        [
            f32::from_bits(0xC40CC000),
            f32::from_bits(0x44438000),
            f32::from_bits(0xC3A00000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x4447C000),
            f32::from_bits(0x43AD8000),
        ],
        [
            f32::from_bits(0xC4108000),
            f32::from_bits(0x4437C000),
            f32::from_bits(0xC35C0000),
        ],
        [
            f32::from_bits(0xC41D8000),
            f32::from_bits(0x444D4000),
            f32::from_bits(0x43C20000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC40A8000),
            f32::from_bits(0x44488000),
            f32::from_bits(0xC3A30000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC4254000),
            f32::from_bits(0x44574000),
            f32::from_bits(0xC3A98000),
        ],
        [
            f32::from_bits(0xC40E8000),
            f32::from_bits(0x44560000),
            f32::from_bits(0xC3998000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC41B0000),
            f32::from_bits(0x44348000),
            f32::from_bits(0xC3CE0000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC40F4000),
            f32::from_bits(0x44378000),
            f32::from_bits(0x439B8000),
        ],
        [
            f32::from_bits(0xC412C000),
            f32::from_bits(0x4454C000),
            f32::from_bits(0xC3C48000),
        ],
    ],
    extra: [
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40000000),
        f32::from_bits(0x40800000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40400000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40E00000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
    ],
    size: [
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42200000),
        f32::from_bits(0x42400000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42480000),
        f32::from_bits(0x42400000),
        f32::from_bits(0x42480000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42480000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42200000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
        f32::from_bits(0x42340000),
    ],
    divisor: [
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x41800000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41800000),
        f32::from_bits(0x41700000),
        f32::from_bits(0x41800000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41800000),
        f32::from_bits(0x41800000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
        f32::from_bits(0x41600000),
    ],
    spread: [
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x40A00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x40A00000),
        f32::from_bits(0x40A00000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x40A00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
    ],
    count: [
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0x28,
        0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0x28, 0x28, 0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0xFFFFFFFF,
        0xFFFFFFFF, 0x28, 0x28, 0xFFFFFFFF, 0x29, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
        0x28, 0x28, 0x28, 0x2A,
    ],
    start: [
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x434B0000),
        f32::from_bits(0x43090000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x43660000),
        f32::from_bits(0x43750000),
        f32::from_bits(0x436E0000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x436E0000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x436E0000),
        f32::from_bits(0x436E0000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x43030000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
        f32::from_bits(0x43640000),
    ],
    middle: [
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43DE0000),
        f32::from_bits(0x43230000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43C30000),
        f32::from_bits(0x44060000),
        f32::from_bits(0x43800000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43800000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43800000),
        f32::from_bits(0x43800000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43C80000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
        f32::from_bits(0x43B18000),
    ],
    end: [
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43DE0000),
        f32::from_bits(0x43CF0000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43520000),
        f32::from_bits(0x43C30000),
        f32::from_bits(0x44060000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43520000),
        f32::from_bits(0x43520000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43520000),
        f32::from_bits(0x43520000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43520000),
        f32::from_bits(0x43C80000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
        f32::from_bits(0x43F18000),
    ],
    high: [
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x460CA000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x460CA000),
        f32::from_bits(0x461C4000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x461C4000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x461C4000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x460CA000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
        f32::from_bits(0x46241000),
    ],
    low: [
        f32::from_bits(0x457A0000),
        f32::from_bits(0x457A0000),
        f32::from_bits(0x457A0000),
        f32::from_bits(0x457A0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x453B8000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x460CA000),
        f32::from_bits(0x461C4000),
        f32::from_bits(0x453B8000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x457A0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x457A0000),
        f32::from_bits(0x453B8000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x460CA000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
        f32::from_bits(0x45A41000),
    ],
};

/// Set 2's tables (IDA 0x933FC8 .. 0x934B78); the first values are the image's.
pub static TABLES_2: KindTables<0x198> = KindTables {
    _gap_0: [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xFA, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFA, 0x44, 0x00,
        0x00, 0x00, 0x00,
    ],
    copied: [const {
        [
            f32::from_bits(0x3F800000),
            f32::from_bits(0x0),
            f32::from_bits(0x0),
        ]
    }; 34],
    from: [
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4460C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4134000),
            f32::from_bits(0x444F4000),
            f32::from_bits(0xC3D98000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC41B0000),
            f32::from_bits(0x444D0000),
            f32::from_bits(0xC3C70000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC4150000),
            f32::from_bits(0x4451C000),
            f32::from_bits(0x439D0000),
        ],
        [
            f32::from_bits(0xC417C000),
            f32::from_bits(0x446DC000),
            f32::from_bits(0xC3A60000),
        ],
    ],
    to: [
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44478000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC4108000),
            f32::from_bits(0x4437C000),
            f32::from_bits(0xC3A90000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0x0),
            f32::from_bits(0xBF800000),
            f32::from_bits(0xBF800000),
        ],
        [
            f32::from_bits(0xC41B0000),
            f32::from_bits(0x44300000),
            f32::from_bits(0xC38D8000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC40F8000),
            f32::from_bits(0x44388000),
            f32::from_bits(0x43D48000),
        ],
        [
            f32::from_bits(0xC412C000),
            f32::from_bits(0x4454C000),
            f32::from_bits(0xC3680000),
        ],
    ],
    extra: [
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40000000),
        f32::from_bits(0x40800000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40400000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40E00000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x0),
        f32::from_bits(0x0),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x40C00000),
    ],
    size: [
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42480000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42680000),
        f32::from_bits(0x42340000),
    ],
    divisor: [
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x41800000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41880000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x41400000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41A80000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41900000),
        f32::from_bits(0x41600000),
    ],
    spread: [
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x40C00000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x44FA0000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
        f32::from_bits(0x41000000),
    ],
    count: [
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0x28, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x2A, 0x28,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x2A,
    ],
    start: [
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x436E0000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x42B40000),
        f32::from_bits(0x43700000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
        f32::from_bits(0x43680000),
    ],
    middle: [
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43F00000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x433E0000),
        f32::from_bits(0x43730000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
    ],
    end: [
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43F00000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43870000),
        f32::from_bits(0x43E40000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
        f32::from_bits(0x43E98000),
    ],
    high: [
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x43020000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x430C0000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
    ],
    low: [
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x43020000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x45FA0000),
        f32::from_bits(0x41200000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
        f32::from_bits(0x42E20000),
    ],
};

/// `TABLES_1`'s place in the image (IDA 0x9332AC).
#[cfg(test)]
pub struct Tables1;

#[cfg(test)]
impl Tables1 {
    pub const START: usize = 0x9332AC;
    pub const LEN: usize = 0xBC4;
}

/// `TABLES_2`'s place in the image (IDA 0x933FC8).
#[cfg(test)]
pub struct Tables2;

#[cfg(test)]
impl Tables2 {
    pub const START: usize = 0x933FC8;
    pub const LEN: usize = 0xBB0;
}

/// Set 1's two outs after its quad (IDA 0x933E70 .. 0x933E7C; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindExtras1 {
    /// +0x0: the kind's `extra` times `extra_k` (`flt_933E70`), the lift along the up
    pub extra: Float,
    /// +0x4: `divisor` copied (`dword_933E74`; the first set only)
    pub divisor_copy: Dword,
    /// +0x8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8: Bytes<4>,
}

impl KindExtras1 {
    /// Set 1's outs; the first values are the image's.
    pub const fn new() -> Self {
        Self {
            extra: Float::from_bits(0x0),
            divisor_copy: Dword::new(0x0),
            _gap_8: Bytes::new([0x00, 0x00, 0x80, 0x47]),
        }
    }
}

impl KindExtras1 {
    pub const START: usize = 0x933E70;
    pub const LEN: usize = 0xC;
}

/// Set 2's out after its quad (IDA 0x934B78 .. 0x934B7C; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindExtra2 {
    /// +0x0: the kind's `extra` times `extra_k` (`flt_934B78`), the lift along the up
    pub extra: Float,
}

impl KindExtra2 {
    /// Set 2's out; the first values are the image's.
    pub const fn new() -> Self {
        Self {
            extra: Float::from_bits(0x0),
        }
    }
}

impl KindExtra2 {
    pub const START: usize = 0x934B78;
    pub const LEN: usize = 0x4;
}

/// A set's constants and light (IDA 0x54 bytes; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct KindConsts {
    /// +0x0: the kind's size is multiplied by it (`kind_values` c0) (`flt_933E7C`)
    pub size_k: f32,
    /// +0x4: the kind's `start` is multiplied by it (c1) (`flt_933E80`)
    pub start_k: f32,
    /// +0x8: the step's last factor (c2) (`flt_933E84`)
    pub step_k: f32,
    /// +0xC: the kind's `middle` is multiplied by it (c3) (`flt_933E88`)
    pub middle_k: f32,
    /// +0x10: the slope's last factor (c4) (`flt_933E8C`)
    pub slope_k: f32,
    /// +0x14: the kind's `spread` is multiplied by it (c5) (`flt_933E90`)
    pub spread_k: f32,
    /// +0x18: the kind's `extra` is multiplied by it (c6) (`flt_933E94`)
    pub extra_k: f32,
    /// +0x1C: the place's blends: for each of the triple's three parts, two factors (c7 .. c12) (`flt_933E98`, `flt_933E9C` ..)
    pub place_k: [f32; 6],
    /// +0x34: what the y part is taken from after the blend (c13) (`flt_933EB0`)
    pub place_less: f32,
    /// +0x38: the light's way, turned by the matrix into `light_turned` (`draw_shadow`) (`flt_933EB4`, `flt_933EB8` ..)
    pub light: [f32; 3],
    /// +0x44: what each corner's depth is lowered by (`draw_shadow`) (`flt_933EC0`, `flt_933EC4` ..)
    pub lifts: [f32; 4],
}

/// Set 1's constants (IDA 0x933E7C .. 0x933ED0); the first values are the image's.
pub static CONSTS_1: KindConsts = KindConsts {
    size_k: f32::from_bits(0x4283126F),
    start_k: f32::from_bits(0x3C8EFA35),
    step_k: f32::from_bits(0x3C8EFA35),
    middle_k: f32::from_bits(0x3C8EFA35),
    slope_k: f32::from_bits(0x3C8EFA35),
    spread_k: f32::from_bits(0x4283126F),
    extra_k: f32::from_bits(0x4283126F),
    place_k: [
        f32::from_bits(0x3F000000),
        f32::from_bits(0xC283126F),
        f32::from_bits(0x3F000000),
        f32::from_bits(0x4283126F),
        f32::from_bits(0x3F000000),
        f32::from_bits(0x4283126F),
    ],
    place_less: f32::from_bits(0x4732A733),
    light: [
        f32::from_bits(0x0),
        f32::from_bits(0x3F800000),
        f32::from_bits(0x0),
    ],
    lifts: [f32::from_bits(0x456475C3); 4],
};

/// Set 2's constants (IDA 0x934B7C .. 0x934BD0); the first values are the image's.
pub static CONSTS_2: KindConsts = KindConsts {
    size_k: f32::from_bits(0x4283126F),
    start_k: f32::from_bits(0x3C8EFA35),
    step_k: f32::from_bits(0x3C8EFA35),
    middle_k: f32::from_bits(0x3C8EFA35),
    slope_k: f32::from_bits(0x3C8EFA35),
    spread_k: f32::from_bits(0x4283126F),
    extra_k: f32::from_bits(0x4283126F),
    place_k: [
        f32::from_bits(0x3F000000),
        f32::from_bits(0xC283126F),
        f32::from_bits(0x3F000000),
        f32::from_bits(0x4283126F),
        f32::from_bits(0x3F000000),
        f32::from_bits(0x4283126F),
    ],
    place_less: f32::from_bits(0x4732A733),
    light: [
        f32::from_bits(0x0),
        f32::from_bits(0x3F800000),
        f32::from_bits(0x0),
    ],
    lifts: [f32::from_bits(0x4525F5C3); 4],
};

/// `CONSTS_1`'s place in the image (IDA 0x933E7C).
#[cfg(test)]
pub struct Consts1;

#[cfg(test)]
impl Consts1 {
    pub const START: usize = 0x933E7C;
    pub const LEN: usize = 0x54;
}

/// `CONSTS_2`'s place in the image (IDA 0x934B7C).
#[cfg(test)]
pub struct Consts2;

#[cfg(test)]
impl Consts2 {
    pub const START: usize = 0x934B7C;
    pub const LEN: usize = 0x54;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_statics_start_as_the_games() {
        let kinds = &effects().kinds;
        let image = crate::moved::image_bytes;
        assert_eq!(size_of::<QuadWork>(), 0xF8);
        assert!(
            bytes(&kinds.quad_1) == image(Quad1::START, Quad1::LEN),
            "QUAD_1"
        );
        assert!(
            bytes(&kinds.quad_2) == image(Quad2::START, Quad2::LEN),
            "QUAD_2"
        );
        assert!(
            bytes(&kinds.quad_3) == image(Quad3::START, Quad3::LEN),
            "QUAD_3"
        );
        assert_eq!(size_of::<KindTables<0x1AC>>(), Tables1::LEN);
        assert_eq!(size_of::<KindTables<0x198>>(), Tables2::LEN);
        assert!(
            bytes(&TABLES_1) == image(Tables1::START, Tables1::LEN),
            "TABLES_1"
        );
        assert!(
            bytes(&TABLES_2) == image(Tables2::START, Tables2::LEN),
            "TABLES_2"
        );
        assert_eq!(size_of::<KindExtras1>(), KindExtras1::LEN);
        assert_eq!(size_of::<KindExtra2>(), KindExtra2::LEN);
        assert!(
            bytes(&kinds.extras_1) == image(KindExtras1::START, KindExtras1::LEN),
            "EXTRAS_1"
        );
        assert!(
            bytes(&kinds.extra_2) == image(KindExtra2::START, KindExtra2::LEN),
            "EXTRA_2"
        );
        assert_eq!(size_of::<KindConsts>(), Consts1::LEN);
        assert!(
            bytes(&CONSTS_1) == image(Consts1::START, Consts1::LEN),
            "CONSTS_1"
        );
        assert!(
            bytes(&CONSTS_2) == image(Consts2::START, Consts2::LEN),
            "CONSTS_2"
        );
    }

    #[test]
    fn the_sets_are_contiguous_in_the_image() {
        assert_eq!(Quad1::START + Quad1::LEN, Tables1::START);
        assert_eq!(Tables1::START + Tables1::LEN, KindExtras1::START);
        assert_eq!(KindExtras1::START + KindExtras1::LEN, Consts1::START);
        assert_eq!(Consts1::START + Consts1::LEN, Quad2::START);
        assert_eq!(Quad2::START + Quad2::LEN, Tables2::START);
        assert_eq!(Tables2::START + Tables2::LEN, KindExtra2::START);
        assert_eq!(KindExtra2::START + KindExtra2::LEN, Consts2::START);
        assert_eq!(Consts2::START + Consts2::LEN, Quad3::START);
    }

    #[test]
    fn the_fields_are_where_the_game_has_them() {
        let q = |base: usize, rel: usize| base + rel;
        assert_eq!(offset_of!(QuadWork, corner), 0x933248 - Quad1::START);
        assert_eq!(offset_of!(QuadWork, frame), 0x933224 - Quad1::START);
        assert_eq!(offset_of!(QuadWork, corners), 0x93327C - Quad1::START);
        assert_eq!(q(Quad2::START, offset_of!(QuadWork, spread)), 0x933F88);
        assert_eq!(q(Quad3::START, offset_of!(QuadWork, corners)), 0x934C98);
        assert_eq!(offset_of!(KindTables<0x1AC>, to), 0x1AC + 0x330);
        assert_eq!(
            Tables1::START + offset_of!(KindTables<0x1AC>, low),
            0x933DE8
        );
        assert_eq!(
            Tables2::START + offset_of!(KindTables<0x198>, count),
            0x934848
        );
        assert_eq!(Consts1::START + offset_of!(KindConsts, lifts), 0x933EC0);
        assert_eq!(Consts2::START + offset_of!(KindConsts, light), 0x934BB4);
    }
}
