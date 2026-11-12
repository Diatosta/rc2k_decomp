//! The marks' kind work, IDA 0x9331B4 ..
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
use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// A quad's work area, the image's layout (IDA 0xF8 bytes, three copies: `QUAD_1`, `QUAD_2`, `QUAD_3`). The offsets are from its start.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct QuadWork {
    /// +0x0: the quad's place (z, y, x as `kind_values` writes it, a reading): the kind's two triples blended (set 1 `flt_9331B4`, `flt_9331B8` ..; set 2 `flt_9331B4`, `flt_9331B8` ..; plane `flt_9331B4`, `flt_9331B8` ..)
    pub place: [PlainFloat; 3],
    /// +0xC: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_c: PlainBytes<4>,
    /// +0x10: the light's way turned by the matrix (`draw_shadow`), a reading (set 1 `flt_9331C4`, `flt_9331C8` ..; set 2 `flt_9331C4`, `flt_9331C8` ..; plane `flt_9331C4`, `flt_9331C8` ..)
    pub light_turned: [PlainFloat; 3],
    /// +0x1C: the matrix turned about z by the angle (`turn_matrix3`); its rows are made the frame's (set 1 `dword_9331D0`, `dword_9331D4` ..; set 2 `dword_9331D0`, `dword_9331D4` ..; plane `dword_9331D0`, `dword_9331D4` ..)
    pub matrix: [PlainFloat; 9],
    /// +0x40: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_40: PlainBytes<12>,
    /// +0x4C: the quad's two sides: the kind's triple (`copied`), then (0, difference y, minus difference x) (set 1 `dword_933200`, `dword_933204` ..; set 2 `dword_933200`, `dword_933204` ..; plane `dword_933200`, `dword_933204` ..)
    pub sides: [[PlainFloat; 3]; 2],
    /// +0x64: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_64: PlainBytes<12>,
    /// +0x70: the sides made one long and square (`normalise_vector3`, `cross_product3`); the light turned is stored in the third (set 1 `dword_933224`, `dword_933228` ..; set 2 `dword_933224`, `dword_933228` ..; plane `dword_933224`, `dword_933228` ..)
    pub frame: [[PlainFloat; 3]; 3],
    /// +0x94: the corner of `g_marksCorners` (-1: none), the kind's `count` (set 1 `dword_933248`; set 2 `dword_933248`; plane `dword_933248`)
    pub corner: PlainDword,
    /// +0x98: the kind's size over its divisor (`kind_values`) (set 1 `flt_93324C`; set 2 `flt_93324C`; plane `flt_93324C`)
    pub ratio: PlainFloat,
    /// +0x9C: the kind's size times its constant (`kind_values`) (set 1 `flt_933250`; set 2 `flt_933250`; plane `flt_933250`)
    pub size: PlainFloat,
    /// +0xA0: the kind's start times its constant (set 1 `flt_933254`; set 2 `flt_933254`; plane `flt_933254`)
    pub start: PlainFloat,
    /// +0xA4: the angle line below the split: the factor (set 1 `flt_933258`; set 2 `flt_933258`; plane `flt_933258`)
    pub step: PlainFloat,
    /// +0xA8: the split of the angle's two lines: the kind's `low` (signed compare of the bits) (set 1 `dword_93325C`; set 2 `dword_93325C`; plane `dword_93325C`)
    pub low: PlainFloat,
    /// +0xAC: the kind's `high` (set 1 `dword_933260`; set 2 `dword_933260`; plane `dword_933260`)
    pub high: PlainFloat,
    /// +0xB0: the angle line above the split: its base (set 1 `flt_933264`; set 2 `flt_933264`; plane `flt_933264`)
    pub middle: PlainFloat,
    /// +0xB4: the angle line above the split: its factor (set 1 `flt_933268`; set 2 `flt_933268`; plane `flt_933268`)
    pub slope: PlainFloat,
    /// +0xB8: the factor the quad is drawn in by (`corners_in`) (set 1 `flt_93326C`; set 2 `flt_93326C`; plane `flt_93326C`)
    pub spread: PlainFloat,
    /// +0xBC: the way between two corners, made one long (`normalise_vector3`) (set 1 `flt_933270`, `flt_933274` ..; set 2 `flt_933270`, `flt_933274` ..; plane `flt_933270`, `flt_933274` ..)
    pub way: [PlainFloat; 3],
    /// +0xC8: the quad's four corners (z, y, x), turned to the view then projected (set 1 `flt_93327C`, `flt_933280` ..; set 2 `flt_93327C`, `flt_933280` ..; plane `flt_93327C`, `flt_933280` ..)
    pub corners: [[PlainFloat; 3]; 4],
}

impl QuadWork {
    /// The second shadow's (`draw_shadow::SECOND`) quad (IDA 0x9331B4 .. 0x9332AC); the first values are the image's.
    pub const fn new_1() -> Self {
        Self {
            place: [
                PlainFloat::new(0.0),
                PlainFloat::new(125536.0),
                PlainFloat::new(0.0),
            ],
            _gap_c: PlainBytes::new([0; 4]),
            light_turned: [const { PlainFloat::new(0.0) }; 3],
            matrix: [const { PlainFloat::new(0.0) }; 9],
            _gap_40: PlainBytes::new([0; 12]),
            sides: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1.0),
                    PlainFloat::new(-1.0),
                ],
            ],
            _gap_64: PlainBytes::new([0; 12]),
            frame: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1.0),
                    PlainFloat::new(-1.0),
                ],
                [const { PlainFloat::new(0.0) }; 3],
            ],
            corner: PlainDword::new(0x4),
            ratio: PlainFloat::new(32768.0),
            size: PlainFloat::new(65536.0),
            start: PlainFloat::new(0.0),
            step: PlainFloat::new(0.0),
            low: PlainFloat::new(0.0),
            high: PlainFloat::new(0.0),
            middle: PlainFloat::new(0.0),
            slope: PlainFloat::new(0.0),
            spread: PlainFloat::new(0.0),
            way: [const { PlainFloat::new(0.0) }; 3],
            corners: [const { [const { PlainFloat::new(0.0) }; 3] }; 4],
        }
    }
}

impl QuadWork {
    /// The first shadow's (`draw_shadow::FIRST`) quad (IDA 0x933ED0 .. 0x933FC8); the first values are the image's.
    pub const fn new_2() -> Self {
        Self {
            place: [
                PlainFloat::new(0.0),
                PlainFloat::new(125536.0),
                PlainFloat::new(0.0),
            ],
            _gap_c: PlainBytes::new([0; 4]),
            light_turned: [const { PlainFloat::new(0.0) }; 3],
            matrix: [const { PlainFloat::new(0.0) }; 9],
            _gap_40: PlainBytes::new([0; 12]),
            sides: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1.0),
                    PlainFloat::new(-1.0),
                ],
            ],
            _gap_64: PlainBytes::new([0; 12]),
            frame: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1.0),
                    PlainFloat::new(-1.0),
                ],
                [const { PlainFloat::new(0.0) }; 3],
            ],
            corner: PlainDword::new(0x4),
            ratio: PlainFloat::new(32768.0),
            size: PlainFloat::new(65536.0),
            start: PlainFloat::new(0.0),
            step: PlainFloat::new(0.0),
            low: PlainFloat::new(0.0),
            high: PlainFloat::new(0.0),
            middle: PlainFloat::new(0.0),
            slope: PlainFloat::new(0.0),
            spread: PlainFloat::new(0.0),
            way: [const { PlainFloat::new(0.0) }; 3],
            corners: [const { [const { PlainFloat::new(0.0) }; 3] }; 4],
        }
    }
}

impl QuadWork {
    /// The plane's (`draw_plane`) quad (IDA 0x934BD0 .. 0x934CC8); the first values are the image's.
    pub const fn new_3() -> Self {
        Self {
            place: [
                PlainFloat::new(0.0),
                PlainFloat::new(125536.0),
                PlainFloat::new(0.0),
            ],
            _gap_c: PlainBytes::new([0; 4]),
            light_turned: [const { PlainFloat::new(0.0) }; 3],
            matrix: [const { PlainFloat::new(0.0) }; 9],
            _gap_40: PlainBytes::new([0; 12]),
            sides: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1.0),
                    PlainFloat::new(-1.0),
                ],
            ],
            _gap_64: PlainBytes::new([0; 12]),
            frame: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1.0),
                    PlainFloat::new(-1.0),
                ],
                [const { PlainFloat::new(0.0) }; 3],
            ],
            corner: PlainDword::new(0x4),
            ratio: PlainFloat::new(32768.0),
            size: PlainFloat::new(65536.0),
            start: PlainFloat::new(0.0),
            step: PlainFloat::new(0.0),
            low: PlainFloat::new(0.0),
            high: PlainFloat::new(0.0),
            middle: PlainFloat::new(0.0),
            slope: PlainFloat::new(0.0),
            spread: PlainFloat::new(0.0),
            way: [const { PlainFloat::new(0.0) }; 3],
            corners: [const { [const { PlainFloat::new(0.0) }; 3] }; 4],
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

/// The per-kind tables of one set (IDA after a leading gap of `GAP` bytes: 3 tables of 0x22 triples, then 10 of 0x22 values). The offsets are from its start.
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
    copied: [const { [1.0, 0.0, 0.0] }; 34],
    from: [
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-528.0, 864.0, -474.0],
        [-607.0, 858.0, -416.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-593.0, 893.0, 249.0],
        [-589.0, 829.0, -317.0],
        [-633.0, 901.0, 302.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-559.0, 874.0, -400.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-678.0, 937.0, -420.0],
        [-605.0, 921.0, -397.0],
        [0.0, -1.0, -1.0],
        [-620.0, 797.0, -488.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-593.0, 828.0, 213.0],
        [-607.0, 951.0, -493.0],
    ],
    to: [
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-499.0, 794.0, -400.0],
        [-563.0, 782.0, -320.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-573.0, 799.0, 347.0],
        [-578.0, 735.0, -220.0],
        [-630.0, 821.0, 388.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-554.0, 802.0, -326.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-661.0, 861.0, -339.0],
        [-570.0, 856.0, -307.0],
        [0.0, -1.0, -1.0],
        [-620.0, 722.0, -412.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-573.0, 734.0, 311.0],
        [-587.0, 851.0, -393.0],
    ],
    extra: [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 0.0, 0.0, 0.0, 3.0, 6.0, 0.0, 0.0, 7.0, 0.0, 0.0,
        6.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    size: [
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 40.0, 48.0, 2000.0, 2000.0, 50.0, 48.0,
        50.0, 2000.0, 2000.0, 50.0, 2000.0, 2000.0, 45.0, 45.0, 2000.0, 40.0, 45.0, 45.0, 45.0,
        45.0, 45.0, 45.0, 45.0, 45.0, 45.0, 45.0, 45.0, 45.0,
    ],
    divisor: [
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 12.0, 16.0, 2000.0, 2000.0, 16.0, 15.0,
        16.0, 2000.0, 2000.0, 18.0, 2000.0, 2000.0, 16.0, 16.0, 2000.0, 12.0, 14.0, 14.0, 14.0,
        14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0,
    ],
    spread: [
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 5.0, 6.0, 2000.0, 2000.0, 6.0, 6.0, 6.0,
        2000.0, 2000.0, 6.0, 2000.0, 2000.0, 5.0, 5.0, 2000.0, 5.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0,
        6.0, 6.0, 6.0, 6.0, 6.0, 6.0,
    ],
    count: [
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0x28,
        0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0x28, 0x28, 0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0xFFFFFFFF,
        0xFFFFFFFF, 0x28, 0x28, 0xFFFFFFFF, 0x29, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
        0x28, 0x28, 0x28, 0x2A,
    ],
    start: [
        90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 203.0, 137.0, 90.0, 90.0, 230.0, 245.0, 238.0, 90.0,
        90.0, 238.0, 90.0, 90.0, 238.0, 238.0, 90.0, 131.0, 228.0, 228.0, 228.0, 228.0, 228.0,
        228.0, 228.0, 228.0, 228.0, 228.0, 228.0, 228.0,
    ],
    middle: [
        190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 444.0, 163.0, 190.0, 190.0, 390.0, 536.0, 256.0,
        190.0, 190.0, 256.0, 190.0, 190.0, 256.0, 256.0, 190.0, 400.0, 355.0, 355.0, 355.0, 355.0,
        355.0, 355.0, 355.0, 355.0, 355.0, 355.0, 355.0, 355.0,
    ],
    end: [
        270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 444.0, 414.0, 270.0, 210.0, 390.0, 536.0, 483.0,
        210.0, 210.0, 483.0, 210.0, 210.0, 483.0, 483.0, 210.0, 400.0, 483.0, 483.0, 483.0, 483.0,
        483.0, 483.0, 483.0, 483.0, 483.0, 483.0, 483.0, 483.0,
    ],
    high: [
        8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 9000.0, 8000.0, 8000.0, 9000.0,
        10000.0, 10500.0, 8000.0, 8000.0, 10000.0, 8000.0, 8000.0, 10000.0, 10500.0, 8000.0,
        9000.0, 10500.0, 10500.0, 10500.0, 10500.0, 10500.0, 10500.0, 10500.0, 10500.0, 10500.0,
        10500.0, 10500.0, 10500.0,
    ],
    low: [
        4000.0, 4000.0, 4000.0, 4000.0, 8000.0, 8000.0, 8000.0, 3000.0, 8000.0, 8000.0, 9000.0,
        10000.0, 3000.0, 8000.0, 8000.0, 4000.0, 8000.0, 8000.0, 4000.0, 3000.0, 8000.0, 9000.0,
        5250.0, 5250.0, 5250.0, 5250.0, 5250.0, 5250.0, 5250.0, 5250.0, 5250.0, 5250.0, 5250.0,
        5250.0,
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
    copied: [const { [1.0, 0.0, 0.0] }; 34],
    from: [
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-596.0, 899.0, 314.0],
        [-589.0, 829.0, -435.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-620.0, 820.0, -398.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-596.0, 839.0, 314.0],
        [-607.0, 951.0, -332.0],
    ],
    to: [
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-574.0, 798.0, 425.0],
        [-578.0, 735.0, -338.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [0.0, -1.0, -1.0],
        [-620.0, 704.0, -283.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-574.0, 738.0, 425.0],
        [-587.0, 851.0, -232.0],
    ],
    extra: [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 0.0, 0.0, 0.0, 3.0, 6.0, 0.0, 0.0, 7.0, 0.0, 0.0,
        6.0, 12.0, 0.0, 0.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0,
    ],
    size: [
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 58.0, 50.0,
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 58.0, 58.0, 58.0,
        58.0, 58.0, 58.0, 58.0, 58.0, 58.0, 58.0, 58.0, 58.0, 45.0,
    ],
    divisor: [
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 12.0, 16.0, 2000.0, 2000.0, 18.0, 17.0,
        12.0, 2000.0, 2000.0, 12.0, 2000.0, 2000.0, 12.0, 12.0, 2000.0, 21.0, 18.0, 18.0, 18.0,
        18.0, 18.0, 18.0, 18.0, 18.0, 18.0, 18.0, 18.0, 14.0,
    ],
    spread: [
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 8.0, 6.0,
        2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 2000.0, 8.0, 8.0, 8.0, 8.0,
        8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0,
    ],
    count: [
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x28, 0x28, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x2A, 0x28,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x2A,
    ],
    start: [
        90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 232.0, 238.0, 90.0, 90.0, 90.0,
        90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 240.0, 232.0, 232.0, 232.0, 232.0, 232.0, 232.0, 232.0,
        232.0, 232.0, 232.0, 232.0, 232.0,
    ],
    middle: [
        190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 467.0, 480.0, 190.0,
        190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 190.0, 243.0, 467.0, 467.0, 467.0, 467.0,
        467.0, 467.0, 467.0, 467.0, 467.0, 467.0, 467.0, 467.0,
    ],
    end: [
        270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 467.0, 480.0, 270.0,
        270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 270.0, 456.0, 467.0, 467.0, 467.0, 467.0,
        467.0, 467.0, 467.0, 467.0, 467.0, 467.0, 467.0, 467.0,
    ],
    high: [
        8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 113.0,
        130.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 140.0,
        113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0,
    ],
    low: [
        8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 113.0,
        130.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 8000.0, 10.0, 113.0,
        113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0, 113.0,
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

/// Set 1's two outs after its quad (IDA 0x933E70 .. 0x933E7C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindExtras1 {
    /// +0x0: the kind's `extra` times `extra_k` (`flt_933E70`), the lift along the up
    pub extra: PlainFloat,
    /// +0x4: `divisor` copied (`dword_933E74`; the first set only)
    pub divisor_copy: PlainDword,
    /// +0x8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8: PlainBytes<4>,
}

impl KindExtras1 {
    /// Set 1's outs; the first values are the image's.
    pub const fn new() -> Self {
        Self {
            extra: PlainFloat::new(0.0),
            divisor_copy: PlainDword::new(0x0),
            _gap_8: PlainBytes::new([0x00, 0x00, 0x80, 0x47]),
        }
    }
}

impl KindExtras1 {
    pub const START: usize = 0x933E70;
    pub const LEN: usize = 0xC;
}

/// Set 2's out after its quad (IDA 0x934B78 .. 0x934B7C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindExtra2 {
    /// +0x0: the kind's `extra` times `extra_k` (`flt_934B78`), the lift along the up
    pub extra: PlainFloat,
}

impl KindExtra2 {
    /// Set 2's out; the first values are the image's.
    pub const fn new() -> Self {
        Self {
            extra: PlainFloat::new(0.0),
        }
    }
}

impl KindExtra2 {
    pub const START: usize = 0x934B78;
    pub const LEN: usize = 0x4;
}

/// A set's constants and light (IDA 0x54 bytes). The offsets are from its start.
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
    size_k: 65.536_f32,
    start_k: 0.017453292_f32,
    step_k: 0.017453292_f32,
    middle_k: 0.017453292_f32,
    slope_k: 0.017453292_f32,
    spread_k: 65.536_f32,
    extra_k: 65.536_f32,
    place_k: [0.5, -65.536, 0.5, 65.536, 0.5, 65.536],
    place_less: 45735.2_f32,
    light: [0.0, 1.0, 0.0],
    lifts: [3655.36_f32; 4],
};

/// Set 2's constants (IDA 0x934B7C .. 0x934BD0); the first values are the image's.
pub static CONSTS_2: KindConsts = KindConsts {
    size_k: 65.536_f32,
    start_k: 0.017453292_f32,
    step_k: 0.017453292_f32,
    middle_k: 0.017453292_f32,
    slope_k: 0.017453292_f32,
    spread_k: 65.536_f32,
    extra_k: 65.536_f32,
    place_k: [0.5, -65.536, 0.5, 65.536, 0.5, 65.536],
    place_less: 45735.2_f32,
    light: [0.0, 1.0, 0.0],
    lifts: [2655.36_f32; 4],
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
        assert_eq!(size_of::<QuadWork>(), 0xF8);
        assert!(
            bytes(&kinds.overlay_quad)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.kinds.overlay_quad/248"
                ),
            "QUAD_1"
        );
        assert!(
            bytes(&kinds.shadow_quad)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.kinds.shadow_quad/248"
                ),
            "QUAD_2"
        );
        assert!(
            bytes(&kinds.plane_quad)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.kinds.plane_quad/248"
                ),
            "QUAD_3"
        );
        assert_eq!(size_of::<KindTables<0x1AC>>(), Tables1::LEN);
        assert_eq!(size_of::<KindTables<0x198>>(), Tables2::LEN);
        assert!(
            bytes(&TABLES_1)
                == crate::test_support::first_values::bytes("effects.kind_store.TABLES_1"),
            "TABLES_1"
        );
        assert!(
            bytes(&TABLES_2)
                == crate::test_support::first_values::bytes("effects.kind_store.TABLES_2"),
            "TABLES_2"
        );
        assert_eq!(size_of::<KindExtras1>(), KindExtras1::LEN);
        assert_eq!(size_of::<KindExtra2>(), KindExtra2::LEN);
        assert!(
            bytes(&kinds.overlay_extras)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.kinds.overlay_extras/12"
                ),
            "EXTRAS_1"
        );
        assert!(
            bytes(&kinds.shadow_extra)
                == crate::test_support::first_values::bytes(
                    "effects.owner.EFFECTS.kinds.shadow_extra/4"
                ),
            "EXTRA_2"
        );
        assert_eq!(size_of::<KindConsts>(), Consts1::LEN);
        assert!(
            bytes(&CONSTS_1)
                == crate::test_support::first_values::bytes("effects.kind_store.CONSTS_1"),
            "CONSTS_1"
        );
        assert!(
            bytes(&CONSTS_2)
                == crate::test_support::first_values::bytes("effects.kind_store.CONSTS_2"),
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
        assert_eq!(offset_of!(QuadWork, corner), 0x94);
        assert_eq!(offset_of!(QuadWork, frame), 0x70);
        assert_eq!(offset_of!(QuadWork, corners), 0xc8);
        assert_eq!(q(Quad2::START, offset_of!(QuadWork, spread)), 0x933F88);
        assert_eq!(q(Quad3::START, offset_of!(QuadWork, corners)), 0x934C98);
        assert_eq!(offset_of!(KindTables<0x1AC>, to), 0x1AC + 0x330);
        assert_eq!(offset_of!(KindTables<0x1AC>, low), 0xb3c);
        assert_eq!(offset_of!(KindTables<0x198>, count), 0x880);
        assert_eq!(offset_of!(KindConsts, lifts), 0x44);
        assert_eq!(offset_of!(KindConsts, light), 0x38);
    }
}
