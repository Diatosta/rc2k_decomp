//! The squares' variables, out of the marks' block (phase 7, typed state): their frame count
//! and rates (`SquareTiming`), curve, circles and frame (`SquareTables`), the 0x800 items
//! (`SquareItems`), and the once-only mark, constants and randoms (`SquareState`).

use crate::session::{Bytes, Dword, Float};

/// The squares' frame count and rates (IDA 0x89CA30 .. 0x89CA3C, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SquareTiming {
    /// 0x89CA30: `dword_89CA30`: the frames the squares have run (`marks::squares_spawn` adds one each call).
    pub frames: Dword,
    /// 0x89CA34: `flt_89CA34`: the squares' rate this frame, the settings' +0x2740 times `SquareState::rate_share` (`marks::squares_frame`, `draw_plane`); 10.0 at first.
    pub rate: Float,
    /// 0x89CA38: `flt_89CA38`: the squares spawned a frame, worked out from the rate by `marks::squares_spawn`.
    pub spawn_rate: Float,
}

pub static SQUARE_TIMING: SquareTiming = SquareTiming {
    frames: Dword::new(0x0),
    rate: Float::from_bits(0x41200000),
    spawn_rate: Float::from_bits(0x0),
};

impl SquareTiming {
    pub const START: usize = 0x89CA30;
    pub const LEN: usize = 0xC;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&SQUARE_TIMING as *const SquareTiming)
    }
}

/// The squares' curve, circles, frame and corners (IDA 0x89CACA .. 0x89D16A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SquareTables {
    /// 0x89CACA: `g_marksSquareCurve`: the quarter-circle curve of 0x100 floats (`marks::set_up_squares`; nothing ported reads it).
    pub curve: [Float; 256],
    /// 0x89CECA: `flt_89CECA` ..: the eight circles' radii the squares' points are made on; the first also scales the axes (`marks::scale_axes`).
    pub radii: [Float; 8],
    /// 0x89CEEA: `flt_89CEEA`: 1 over the view's farthest depth (`marks::squares_setup`, `begin`s of `draw_pong_box`, `draw_square`).
    pub inverse: Float,
    /// 0x89CEEE: `flt_89CEEE`, `flt_89CEF2`: the first and second axes' lengths over the frame's divisors (`marks::squares_frame`, `draw_plane`).
    pub lengths: [Float; 2],
    /// 0x89CEF6: `flt_89CEF6` ..: the frame's place (the settings' +0x2720 .., as floats).
    pub place: [Float; 3],
    /// 0x89CF02: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_89cf02: Bytes<24>,
    /// 0x89CF1A: `flt_89CF1A` ..: the frame's three axes (the first and second made from the settings, the third their cross product), each (x, y, z), made one long.
    pub axes: [[Float; 3]; 3],
    /// 0x89CF3E: `flt_89CF3E` ..: the first two axes times the first radius (`marks::scale_axes`).
    pub scaled_by_radius: [[Float; 3]; 2],
    /// 0x89CF56: `flt_89CF56` ..: the first two axes scaled by the per-part halves (`marks::scale_axes`).
    pub half_axes: [[Float; 3]; 2],
    /// 0x89CF6E: `flt_89CF6E` ..: the first two axes scaled by the lengths and the per-part factors (`marks::scale_axes`).
    pub scaled_axes: [[Float; 3]; 2],
    /// 0x89CF86: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_89cf86: Bytes<12>,
    /// 0x89CF92: `flt_89CF92` .. `flt_89CFBE`: the four corners (x, y, depth) of the square being drawn.
    pub corners: [[Float; 3]; 4],
    /// 0x89CFC2: Not named: 36 bytes (a gap, or unreferenced data).
    pub _gap_89cfc2: Bytes<36>,
    /// 0x89CFE6: `g_marksSquarePoints`: for each of the eight radii the circle of four points (x, y, and a third float the port does not use).
    pub points: [[[Float; 3]; 4]; 8],
    /// 0x89D166: `dword_89D166`: an offset into the items, where `marks::spawn_square` looks for a free one.
    pub cursor: Dword,
}

pub static SQUARE_TABLES: SquareTables = SquareTables {
    curve: [const { Float::from_bits(0x0) }; 256],
    radii: [
        Float::from_bits(0x449E0000),
        Float::from_bits(0x458EA000),
        Float::from_bits(0x458B8000),
        Float::from_bits(0x45886000),
        Float::from_bits(0x45854000),
        Float::from_bits(0x45822000),
        Float::from_bits(0x45898000),
        Float::from_bits(0x45802000),
    ],
    inverse: Float::from_bits(0x0),
    lengths: [Float::from_bits(0x42166666), Float::from_bits(0x42166666)],
    place: [
        Float::from_bits(0xC7666666),
        Float::from_bits(0xC6E66666),
        Float::from_bits(0x480E2000),
    ],
    _gap_89cf02: Bytes::new([
        0x66, 0x66, 0x66, 0xC7, 0x66, 0x66, 0xE6, 0xC6, 0x00, 0x00, 0x80, 0x47, 0x00, 0x00, 0x80,
        0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
    axes: [
        [
            Float::from_bits(0x3F800000),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
        [
            Float::from_bits(0x0),
            Float::from_bits(0x3F800000),
            Float::from_bits(0x0),
        ],
        [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x3F800000),
        ],
    ],
    scaled_by_radius: [
        [
            Float::from_bits(0x44D00000),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
        [
            Float::from_bits(0x0),
            Float::from_bits(0xC4948000),
            Float::from_bits(0x44948000),
        ],
    ],
    half_axes: [
        [
            Float::from_bits(0x44D00000),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
        [
            Float::from_bits(0x0),
            Float::from_bits(0xC4948000),
            Float::from_bits(0x44948000),
        ],
    ],
    scaled_axes: [
        [
            Float::from_bits(0x3F800000),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
        [
            Float::from_bits(0x0),
            Float::from_bits(0xBF3504E6),
            Float::from_bits(0x3F3504E6),
        ],
    ],
    _gap_89cf86: Bytes::new([0; 12]),
    corners: [const {
        [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ]
    }; 4],
    _gap_89cfc2: Bytes::new([0; 36]),
    points: [const {
        [const {
            [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ]
        }; 4]
    }; 8],
    cursor: Dword::new(0x0),
};

impl SquareTables {
    pub const START: usize = 0x89CACA;
    pub const LEN: usize = 0x6A0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&SQUARE_TABLES as *const SquareTables)
    }
}

/// One square item (0x3C bytes; `marks::spawn_square`, `expire_squares`, `run_squares`).
#[repr(C)]
pub struct SquareItem {
    /// +0: 1 while the square lives, 0 for a free item.
    pub live: Dword,
    /// +4: the scene clock when it was made.
    pub born: Dword,
    /// +8: its age in frame ticks.
    pub age: Dword,
    /// +0xC: its size (from its kind; also added to its texture places).
    pub size: Float,
    /// +0x10: a second size (0 at first; also added to its texture places).
    pub size_2: Float,
    /// +0x14: three dwords, 0.
    pub spare_14: [Dword; 3],
    /// +0x20, +0x24: its place (x, z) as floats.
    pub place: [Float; 2],
    /// +0x28: five dwords, 0.
    pub spare_28: [Dword; 5],
}

impl SquareItem {
    #[cfg(test)]
    pub const LEN: usize = 0x3C;
    pub const fn new() -> Self {
        SquareItem {
            live: Dword::new(0),
            born: Dword::new(0),
            age: Dword::new(0),
            size: Float::from_bits(0),
            size_2: Float::from_bits(0),
            spare_14: [const { Dword::new(0) }; 3],
            place: [const { Float::from_bits(0) }; 2],
            spare_28: [const { Dword::new(0) }; 5],
        }
    }
}

/// The squares' 0x800 items (`g_marksSquareItems`) (IDA 0x89E18A .. 0x8BC18A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SquareItems {
    /// 0x89E18A: `g_marksSquareItems`: the items, 0x3C bytes each.
    pub items: [SquareItem; 0x800],
}

pub static SQUARE_ITEMS: SquareItems = SquareItems {
    items: [const { SquareItem::new() }; 0x800],
};

impl SquareItems {
    pub const START: usize = 0x89E18A;
    pub const LEN: usize = 0x1E000;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&SQUARE_ITEMS as *const SquareItems)
    }
}

/// The squares' once-only mark, constants and randoms (IDA 0x8BC18A .. 0x8BC266, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SquareState {
    /// 0x8BC18A: `g_marksSquaresDone`: 0, or where `marks::set_up_squares` goes on.
    pub done: Dword,
    /// 0x8BC18E: `flt_8BC18E`, `flt_8BC192`: the curve's two constants (1024.0, 0.0005).
    pub curve_ends: [Float; 2],
    /// 0x8BC196: `dword_8BC196`: the circles' steps (4).
    pub steps: Dword,
    /// 0x8BC19A: `flt_8BC19A`: the circles' angle (2.0).
    pub step_angle: Float,
    /// 0x8BC19E: `flt_8BC19E`: what the settings' rate is multiplied by (1.0; `marks::squares_frame`).
    pub rate_share: Float,
    /// 0x8BC1A2: `flt_8BC1A2`, `flt_8BC1A6`: the axes' lengths are divided by them (2048.0, 1024.0).
    pub length_divisors: [Float; 2],
    /// 0x8BC1AA: `flt_8BC1AA`: the rate times it must come to a whole number above 0 for the squares to run (65536.0).
    pub share: Float,
    /// 0x8BC1AE: `dword_8BC1AE` .. `dword_8BC1BA`: the texture places of a drawn square (u a, u c, v b, v d as floats' bits; `marks::draw_square`).
    pub places: [Dword; 4],
    /// 0x8BC1BE: `flt_8BC1BE` .. `flt_8BC1CA`: 4096.0 four times (the port does not read them).
    pub unread_floats: [Float; 4],
    /// 0x8BC1CE: `dword_8BC1CE` .. `dword_8BC1DA`: 0, 0.5, 0.99999, 0.99999 (the port does not read them).
    pub unread_dwords: [Dword; 4],
    /// 0x8BC1DE: `flt_8BC1DE` .. `flt_8BC1F2`: the axes' per-part factors, 2048.0 three times and 1024.0 three times (`marks::scale_axes`).
    pub axis_factors: [Float; 6],
    /// 0x8BC1F6: `flt_8BC1F6` .. `flt_8BC20A`: 0.5 six times (`marks::scale_axes`).
    pub halves: [Float; 6],
    /// 0x8BC20E: `dword_8BC20E`: 256.0 (the port does not read it).
    pub unread_rhw: Dword,
    /// 0x8BC212: `dword_8BC212` .. `dword_8BC21E`: the texture places `marks::squares_setup` puts on the vertices.
    pub setup_places: [Dword; 4],
    /// 0x8BC222: `dword_8BC222` .. `dword_8BC22E`: the texture places `marks::run_squares` puts on the vertices.
    pub run_places: [Dword; 4],
    /// 0x8BC232: `dword_8BC232`: the age over which a square fades (16).
    pub fade: Dword,
    /// 0x8BC236: `flt_8BC236`: 1024.0, what a square's z is taken from (`marks::run_squares`).
    pub far: Float,
    /// 0x8BC23A: `flt_8BC23A`, `flt_8BC23E`, `flt_8BC242`: a new square's size is its kind times the first plus the second; the third is its +0x10 (`marks::spawn_square`).
    pub size_k: [Float; 3],
    /// 0x8BC246: `flt_8BC246`, `flt_8BC24A`: 0.1 and 0 (`marks::squares_spawn`).
    pub rate_constants: [Float; 2],
    /// 0x8BC24E: `flt_8BC24E`: 65536.0, what the rate's fraction is multiplied by (`marks::squares_spawn`).
    pub share_k: Float,
    /// 0x8BC252: `dword_8BC252`: the random (times 0xC5 plus 0x5D) of whether one more square is spawned.
    pub spawn_random: Dword,
    /// 0x8BC256: `dword_8BC256`: the random of a square's place and kind (`marks::pick_square`).
    pub pick_random: Dword,
    /// 0x8BC25A: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bc25a: Bytes<4>,
    /// 0x8BC25E: `flt_8BC25E`: the settings' +0x2750 times `EdgeSettings::factor` (`marks::scale_setting`).
    pub scaled_setting: Float,
    /// 0x8BC262: `flt_8BC262`: 50.0 and a bit (the port does not read it).
    pub unread_limit: Float,
}

pub static SQUARE_STATE: SquareState = SquareState {
    done: Dword::new(0x0),
    curve_ends: [Float::from_bits(0x44800000), Float::from_bits(0x3A0EA4A9)],
    steps: Dword::new(0x4),
    step_angle: Float::from_bits(0x40000000),
    rate_share: Float::from_bits(0x3F800000),
    length_divisors: [Float::from_bits(0x45000000), Float::from_bits(0x44800000)],
    share: Float::from_bits(0x47800000),
    places: [
        Dword::new(0x3B000000),
        Dword::new(0x3B000000),
        Dword::new(0x3D840000),
        Dword::new(0x3D840000),
    ],
    unread_floats: [
        Float::from_bits(0x46800000),
        Float::from_bits(0x46800000),
        Float::from_bits(0x46800000),
        Float::from_bits(0x46800000),
    ],
    unread_dwords: [
        Dword::new(0x0),
        Dword::new(0x3F000000),
        Dword::new(0x3F7FFFFE),
        Dword::new(0x3F7FFFFE),
    ],
    axis_factors: [
        Float::from_bits(0x45000000),
        Float::from_bits(0x45000000),
        Float::from_bits(0x45000000),
        Float::from_bits(0x44800000),
        Float::from_bits(0x44800000),
        Float::from_bits(0x44800000),
    ],
    halves: [const { Float::from_bits(0x3F000000) }; 6],
    unread_rhw: Dword::new(0x43800000),
    setup_places: [
        Dword::new(0x3DE80000),
        Dword::new(0x3D000000),
        Dword::new(0x3E7C0000),
        Dword::new(0x3E280000),
    ],
    run_places: [
        Dword::new(0x3B000000),
        Dword::new(0x3B000000),
        Dword::new(0x3D840000),
        Dword::new(0x3D840000),
    ],
    fade: Dword::new(0x10),
    far: Float::from_bits(0x44800000),
    size_k: [
        Float::from_bits(0x3D800000),
        Float::from_bits(0x3B000000),
        Float::from_bits(0x0),
    ],
    rate_constants: [Float::from_bits(0x3DCCCCCD), Float::from_bits(0x0)],
    share_k: Float::from_bits(0x47800000),
    spawn_random: Dword::new(0x0),
    pick_random: Dword::new(0x0),
    _gap_8bc25a: Bytes::new([0; 4]),
    scaled_setting: Float::from_bits(0x41200000),
    unread_limit: Float::from_bits(0x4248001A),
};

impl SquareState {
    pub const START: usize = 0x8BC18A;
    pub const LEN: usize = 0xDC;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&SQUARE_STATE as *const SquareState)
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn square_timing_starts_as_the_games() {
        assert_eq!(size_of::<SquareTiming>(), SquareTiming::LEN);
        let image = crate::moved::image_bytes(SquareTiming::START, SquareTiming::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SquareTiming::home()) as *const u8,
                SquareTiming::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "SQUARE_TIMING starts differently from the image"
        );
        assert_eq!(
            offset_of!(SquareTiming, rate),
            0x89CA34 - SquareTiming::START
        );
        assert_eq!(
            offset_of!(SquareTiming, spawn_rate),
            0x89CA38 - SquareTiming::START
        );
    }

    #[test]
    fn square_tables_starts_as_the_games() {
        assert_eq!(size_of::<SquareTables>(), SquareTables::LEN);
        let image = crate::moved::image_bytes(SquareTables::START, SquareTables::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SquareTables::home()) as *const u8,
                SquareTables::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "SQUARE_TABLES starts differently from the image"
        );
        assert_eq!(
            offset_of!(SquareTables, radii),
            0x89CECA - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, inverse),
            0x89CEEA - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, lengths),
            0x89CEEE - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, place),
            0x89CEF6 - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, axes),
            0x89CF1A - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, scaled_by_radius),
            0x89CF3E - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, half_axes),
            0x89CF56 - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, scaled_axes),
            0x89CF6E - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, corners),
            0x89CF92 - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, points),
            0x89CFE6 - SquareTables::START
        );
        assert_eq!(
            offset_of!(SquareTables, cursor),
            0x89D166 - SquareTables::START
        );
    }

    #[test]
    fn square_items_starts_as_the_games() {
        assert_eq!(size_of::<SquareItems>(), SquareItems::LEN);
        let image = crate::moved::image_bytes(SquareItems::START, SquareItems::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SquareItems::home()) as *const u8,
                SquareItems::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "SQUARE_ITEMS starts differently from the image"
        );
        assert_eq!(size_of::<SquareItem>(), SquareItem::LEN);
    }

    #[test]
    fn square_state_starts_as_the_games() {
        assert_eq!(size_of::<SquareState>(), SquareState::LEN);
        let image = crate::moved::image_bytes(SquareState::START, SquareState::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SquareState::home()) as *const u8,
                SquareState::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "SQUARE_STATE starts differently from the image"
        );
        assert_eq!(
            offset_of!(SquareState, curve_ends),
            0x8BC18E - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, steps),
            0x8BC196 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, rate_share),
            0x8BC19E - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, length_divisors),
            0x8BC1A2 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, share),
            0x8BC1AA - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, places),
            0x8BC1AE - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, unread_floats),
            0x8BC1BE - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, axis_factors),
            0x8BC1DE - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, halves),
            0x8BC1F6 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, setup_places),
            0x8BC212 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, run_places),
            0x8BC222 - SquareState::START
        );
        assert_eq!(offset_of!(SquareState, fade), 0x8BC232 - SquareState::START);
        assert_eq!(offset_of!(SquareState, far), 0x8BC236 - SquareState::START);
        assert_eq!(
            offset_of!(SquareState, size_k),
            0x8BC23A - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, rate_constants),
            0x8BC246 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, share_k),
            0x8BC24E - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, spawn_random),
            0x8BC252 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, pick_random),
            0x8BC256 - SquareState::START
        );
        assert_eq!(
            offset_of!(SquareState, scaled_setting),
            0x8BC25E - SquareState::START
        );
    }
}
