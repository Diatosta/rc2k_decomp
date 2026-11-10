//! The windscreen flakes' variables, out of the marks' block (phase 7, typed state): their frame count
//! and rates (`WindscreenFlakeTiming`), curve, circles and frame (`WindscreenFlakeTables`), the 0x800 items
//! (`WindscreenFlakeItems`), and the once-only mark, constants and randoms (`WindscreenFlakeState`).

use crate::session::{Bytes, Dword, Float};

/// The windscreen flakes' frame count and rates (IDA 0x89CA30 .. 0x89CA3C, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeTiming {
    /// 0x89CA30: `dword_89CA30`: the frames the windscreen flakes have run (`effects::windscreen_flakes_spawn` adds one each call).
    pub frames: Dword,
    /// 0x89CA34: `flt_89CA34`: the windscreen flakes' rate this frame, the settings' +0x2740 times `WindscreenFlakeState::rate_share` (`effects::windscreen_flakes_frame`, `draw_plane`); 10.0 at first.
    pub rate: Float,
    /// 0x89CA38: `flt_89CA38`: the windscreen flakes spawned a frame, worked out from the rate by `effects::windscreen_flakes_spawn`.
    pub spawn_rate: Float,
}

impl WindscreenFlakeTiming {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeTiming {
            frames: Dword::new(0x0),
            rate: Float::from_bits(0x41200000),
            spawn_rate: Float::from_bits(0x0),
        }
    }
    pub const START: usize = 0x89CA30;
    pub const LEN: usize = 0xC;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(
            &raw const crate::effects::owner::EFFECTS.windscreen_flakes.timing,
        ) as usize
    }
}

/// The windscreen flakes' curve, circles, frame and corners (IDA 0x89CACA .. 0x89D16A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeTables {
    /// 0x89CACA: `g_marksWindscreenFlakeCurve`: the quarter-circle curve of 0x100 floats (`effects::set_up_windscreen_flakes`; nothing ported reads it).
    pub curve: [Float; 256],
    /// 0x89CECA: `flt_89CECA` ..: the eight circles' radii the windscreen flakes' points are made on; the first also scales the axes (`effects::scale_axes`).
    pub radii: [Float; 8],
    /// 0x89CEEA: `flt_89CEEA`: 1 over the view's farthest depth (`effects::windscreen_flakes_setup`, `begin`s of `draw_pong_box`, `draw_windscreen_flake`).
    pub inverse: Float,
    /// 0x89CEEE: `flt_89CEEE`, `flt_89CEF2`: the first and second axes' lengths over the frame's divisors (`effects::windscreen_flakes_frame`, `draw_plane`).
    pub lengths: [Float; 2],
    /// 0x89CEF6: `flt_89CEF6` ..: the frame's place (the settings' +0x2720 .., as floats).
    pub place: [Float; 3],
    /// 0x89CF02: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_89cf02: Bytes<24>,
    /// 0x89CF1A: `flt_89CF1A` ..: the frame's three axes (the first and second made from the settings, the third their cross product), each (x, y, z), made one long.
    pub axes: [[Float; 3]; 3],
    /// 0x89CF3E: `flt_89CF3E` ..: the first two axes times the first radius (`effects::scale_axes`).
    pub scaled_by_radius: [[Float; 3]; 2],
    /// 0x89CF56: `flt_89CF56` ..: the first two axes scaled by the per-part halves (`effects::scale_axes`).
    pub half_axes: [[Float; 3]; 2],
    /// 0x89CF6E: `flt_89CF6E` ..: the first two axes scaled by the lengths and the per-part factors (`effects::scale_axes`).
    pub scaled_axes: [[Float; 3]; 2],
    /// 0x89CF86: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_89cf86: Bytes<12>,
    /// 0x89CF92: `flt_89CF92` .. `flt_89CFBE`: the four corners (x, y, depth) of the flake being drawn.
    pub corners: [[Float; 3]; 4],
    /// 0x89CFC2: Not named: 36 bytes (a gap, or unreferenced data).
    pub _gap_89cfc2: Bytes<36>,
    /// 0x89CFE6: `g_marksWindscreenFlakePoints`: for each of the eight radii the circle of four points (x, y, and a third float the port does not use).
    pub points: [[[Float; 3]; 4]; 8],
    /// 0x89D166: `dword_89D166`: an offset into the items, where `effects::spawn_windscreen_flake` looks for a free one.
    pub cursor: Dword,
}

impl WindscreenFlakeTables {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeTables {
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
                0x66, 0x66, 0x66, 0xC7, 0x66, 0x66, 0xE6, 0xC6, 0x00, 0x00, 0x80, 0x47, 0x00, 0x00,
                0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
        }
    }
    pub const START: usize = 0x89CACA;
    pub const LEN: usize = 0x6A0;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(
            &raw const crate::effects::owner::EFFECTS.windscreen_flakes.tables,
        ) as usize
    }
}

/// One windscreen flake item (0x3C bytes; `effects::spawn_windscreen_flake`, `expire_windscreen_flakes`, `run_windscreen_flakes`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeItem {
    /// +0: 1 while the flake lives, 0 for a free item.
    pub live: Dword,
    /// +4: the scene clock when it was made.
    pub born: Dword,
    /// +8: its age in frame ticks.
    pub age: Dword,
    /// +0xC: the flake's texture u offset: its kind (0 .. 7) / 16 + 1/512, the tile across the sheet (set at spawn, added to the four vertices' u by `run_windscreen_flakes`).
    pub texture_u: Float,
    /// +0x10: the flake's texture v offset: 0 (from the third constant of `tile_offset_k`), added to the four vertices' v.
    pub texture_v: Float,
    /// +0x14: three dwords, always 0 (the original set them at spawn; nothing read them, so they are no longer written).
    pub spare_14: [Dword; 3],
    /// +0x20, +0x24: its place (x, z) as floats.
    pub place: [Float; 2],
    /// +0x28: five dwords, always 0 (the original set them at spawn; nothing read them, so they are no longer written).
    pub spare_28: [Dword; 5],
}

impl WindscreenFlakeItem {
    #[cfg(test)]
    pub const LEN: usize = 0x3C;
    pub const fn new() -> Self {
        WindscreenFlakeItem {
            live: Dword::new(0),
            born: Dword::new(0),
            age: Dword::new(0),
            texture_u: Float::from_bits(0),
            texture_v: Float::from_bits(0),
            spare_14: [const { Dword::new(0) }; 3],
            place: [const { Float::from_bits(0) }; 2],
            spare_28: [const { Dword::new(0) }; 5],
        }
    }
}

/// The windscreen flakes' 0x800 items (`g_marksWindscreenFlakeItems`) (IDA 0x89E18A .. 0x8BC18A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeItems {
    /// 0x89E18A: `g_marksWindscreenFlakeItems`: the items, 0x3C bytes each.
    pub items: [WindscreenFlakeItem; 0x800],
}

impl WindscreenFlakeItems {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeItems {
            items: [const { WindscreenFlakeItem::new() }; 0x800],
        }
    }
    pub const START: usize = 0x89E18A;
    pub const LEN: usize = 0x1E000;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(
            &raw const crate::effects::owner::EFFECTS.windscreen_flakes.items,
        ) as usize
    }
}

/// The windscreen flakes' once-only mark, constants and randoms (IDA 0x8BC18A .. 0x8BC266, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeState {
    /// 0x8BC18A: `g_marksWindscreenFlakesDone`: 0, or where `effects::set_up_windscreen_flakes` goes on.
    pub done: Dword,
    /// 0x8BC18E: `flt_8BC18E`, `flt_8BC192`: the curve's two constants (1024.0, 0.0005).
    pub curve_ends: [Float; 2],
    /// 0x8BC196: `dword_8BC196`: the circles' steps (4).
    pub steps: Dword,
    /// 0x8BC19A: `flt_8BC19A`: the circles' angle (2.0).
    pub step_angle: Float,
    /// 0x8BC19E: `flt_8BC19E`: what the settings' rate is multiplied by (1.0; `effects::windscreen_flakes_frame`).
    pub rate_share: Float,
    /// 0x8BC1A2: `flt_8BC1A2`, `flt_8BC1A6`: the axes' lengths are divided by them (2048.0, 1024.0).
    pub length_divisors: [Float; 2],
    /// 0x8BC1AA: `flt_8BC1AA`: the rate times it must come to a whole number above 0 for the windscreen flakes to run (65536.0).
    pub share: Float,
    /// 0x8BC1AE: `dword_8BC1AE` .. `dword_8BC1BA`: the texture places of a drawn flake (u a, u c, v b, v d as floats' bits; `effects::draw_windscreen_flake`).
    pub places: [Dword; 4],
    /// 0x8BC1BE: `flt_8BC1BE` .. `flt_8BC1CA`: 4096.0 four times (the port does not read them).
    pub unread_floats: [Float; 4],
    /// 0x8BC1CE: `dword_8BC1CE` .. `dword_8BC1DA`: 0, 0.5, 0.99999, 0.99999 (the port does not read them).
    pub unread_dwords: [Dword; 4],
    /// 0x8BC1DE: `flt_8BC1DE` .. `flt_8BC1F2`: the axes' per-part factors, 2048.0 three times and 1024.0 three times (`effects::scale_axes`).
    pub axis_factors: [Float; 6],
    /// 0x8BC1F6: `flt_8BC1F6` .. `flt_8BC20A`: 0.5 six times (`effects::scale_axes`).
    pub halves: [Float; 6],
    /// 0x8BC20E: `dword_8BC20E`: 256.0 (the port does not read it).
    pub unread_rhw: Dword,
    /// 0x8BC212: `dword_8BC212` .. `dword_8BC21E`: the texture places `effects::windscreen_flakes_setup` puts on the vertices.
    pub setup_places: [Dword; 4],
    /// 0x8BC222: `dword_8BC222` .. `dword_8BC22E`: the texture places `effects::run_windscreen_flakes` puts on the vertices.
    pub run_places: [Dword; 4],
    /// 0x8BC232: `dword_8BC232`: the age over which a flake fades (16).
    pub fade: Dword,
    /// 0x8BC236: `flt_8BC236`: 1024.0, what a flake's z is taken from (`effects::run_windscreen_flakes`).
    pub far: Float,
    /// 0x8BC23A: `flt_8BC23A`, `flt_8BC23E`, `flt_8BC242`: a new flake's texture u offset is its kind times the first (1/16) plus the second (1/512); the third (0) is its v offset (+0x10) (`effects::spawn_windscreen_flake`).
    pub tile_offset_k: [Float; 3],
    /// 0x8BC246: `flt_8BC246`, `flt_8BC24A`: 0.1 and 0 (`effects::windscreen_flakes_spawn`).
    pub rate_constants: [Float; 2],
    /// 0x8BC24E: `flt_8BC24E`: 65536.0, what the rate's fraction is multiplied by (`effects::windscreen_flakes_spawn`).
    pub share_k: Float,
    /// 0x8BC252: `dword_8BC252`: the random (times 0xC5 plus 0x5D) of whether one more flake is spawned.
    pub spawn_random: Dword,
    /// 0x8BC256: `dword_8BC256`: the random of a flake's place and kind (`effects::pick_windscreen_flake`).
    pub pick_random: Dword,
    /// 0x8BC25A: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8bc25a: Bytes<4>,
    /// 0x8BC25E: `flt_8BC25E`: the settings' +0x2750 times `EdgeSettings::factor` (`effects::scale_setting`).
    pub scaled_setting: Float,
    /// 0x8BC262: `flt_8BC262`: 50.0 and a bit (the port does not read it).
    pub unread_limit: Float,
}

impl WindscreenFlakeState {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeState {
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
            tile_offset_k: [
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
        }
    }
    pub const START: usize = 0x8BC18A;
    pub const LEN: usize = 0xDC;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(
            &raw const crate::effects::owner::EFFECTS.windscreen_flakes.state,
        ) as usize
    }
}

/// The windscreen flakes' state, one part of the effects' owner (`effects::owner::Effects`).
/// The pong box and the plane draw with its frame and corners too.
pub struct WindscreenFlakes {
    pub timing: WindscreenFlakeTiming,
    pub tables: WindscreenFlakeTables,
    pub items: WindscreenFlakeItems,
    pub state: WindscreenFlakeState,
}

impl WindscreenFlakes {
    pub const fn new() -> Self {
        WindscreenFlakes {
            timing: WindscreenFlakeTiming::new(),
            tables: WindscreenFlakeTables::new(),
            items: WindscreenFlakeItems::new(),
            state: WindscreenFlakeState::new(),
        }
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn windscreen_flake_timing_starts_as_the_games() {
        assert_eq!(
            size_of::<WindscreenFlakeTiming>(),
            WindscreenFlakeTiming::LEN
        );
        let image =
            crate::moved::image_bytes(WindscreenFlakeTiming::START, WindscreenFlakeTiming::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(WindscreenFlakeTiming::home()).cast_const(),
                WindscreenFlakeTiming::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "WINDSCREEN_FLAKE_TIMING starts differently from the image"
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTiming, rate),
            0x89CA34 - WindscreenFlakeTiming::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTiming, spawn_rate),
            0x89CA38 - WindscreenFlakeTiming::START
        );
    }

    #[test]
    fn windscreen_flake_tables_starts_as_the_games() {
        assert_eq!(
            size_of::<WindscreenFlakeTables>(),
            WindscreenFlakeTables::LEN
        );
        let image =
            crate::moved::image_bytes(WindscreenFlakeTables::START, WindscreenFlakeTables::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(WindscreenFlakeTables::home()).cast_const(),
                WindscreenFlakeTables::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "WINDSCREEN_FLAKE_TABLES starts differently from the image"
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, radii),
            0x89CECA - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, inverse),
            0x89CEEA - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, lengths),
            0x89CEEE - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, place),
            0x89CEF6 - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, axes),
            0x89CF1A - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, scaled_by_radius),
            0x89CF3E - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, half_axes),
            0x89CF56 - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, scaled_axes),
            0x89CF6E - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, corners),
            0x89CF92 - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, points),
            0x89CFE6 - WindscreenFlakeTables::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeTables, cursor),
            0x89D166 - WindscreenFlakeTables::START
        );
    }

    #[test]
    fn windscreen_flake_items_starts_as_the_games() {
        assert_eq!(size_of::<WindscreenFlakeItems>(), WindscreenFlakeItems::LEN);
        let image =
            crate::moved::image_bytes(WindscreenFlakeItems::START, WindscreenFlakeItems::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(WindscreenFlakeItems::home()).cast_const(),
                WindscreenFlakeItems::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "WINDSCREEN_FLAKE_ITEMS starts differently from the image"
        );
        assert_eq!(size_of::<WindscreenFlakeItem>(), WindscreenFlakeItem::LEN);
    }

    #[test]
    fn windscreen_flake_state_starts_as_the_games() {
        assert_eq!(size_of::<WindscreenFlakeState>(), WindscreenFlakeState::LEN);
        let image =
            crate::moved::image_bytes(WindscreenFlakeState::START, WindscreenFlakeState::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(WindscreenFlakeState::home()).cast_const(),
                WindscreenFlakeState::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "WINDSCREEN_FLAKE_STATE starts differently from the image"
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, curve_ends),
            0x8BC18E - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, steps),
            0x8BC196 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, rate_share),
            0x8BC19E - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, length_divisors),
            0x8BC1A2 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, share),
            0x8BC1AA - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, places),
            0x8BC1AE - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, unread_floats),
            0x8BC1BE - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, axis_factors),
            0x8BC1DE - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, halves),
            0x8BC1F6 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, setup_places),
            0x8BC212 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, run_places),
            0x8BC222 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, fade),
            0x8BC232 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, far),
            0x8BC236 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, tile_offset_k),
            0x8BC23A - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, rate_constants),
            0x8BC246 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, share_k),
            0x8BC24E - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, spawn_random),
            0x8BC252 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, pick_random),
            0x8BC256 - WindscreenFlakeState::START
        );
        assert_eq!(
            offset_of!(WindscreenFlakeState, scaled_setting),
            0x8BC25E - WindscreenFlakeState::START
        );
    }
}
