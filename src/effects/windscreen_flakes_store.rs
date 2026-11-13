//! The windscreen flakes' variables: their frame count
//! and rates (`WindscreenFlakeTiming`), curve, circles and frame (`WindscreenFlakeTables`), the 0x800 items
//! (`WindscreenFlakeItems`), and the once-only mark, constants and randoms (`WindscreenFlakeState`).

use crate::session::{PlainDword, PlainFloat};

/// The windscreen flakes' frame count and rates (IDA 0x89CA30 .. 0x89CA3C).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeTiming {
    /// 0x89CA30: `dword_89CA30`: the frames the windscreen flakes have run (`effects::windscreen_flakes_spawn` adds one each call).
    pub frames: PlainDword,
    /// 0x89CA34: `flt_89CA34`: the windscreen flakes' rate this frame, the settings' +0x2740 times `WindscreenFlakeState::rate_share` (`effects::windscreen_flakes_frame`, `draw_plane`); 10.0 at first.
    pub rate: PlainFloat,
    /// 0x89CA38: `flt_89CA38`: the windscreen flakes spawned a frame, worked out from the rate by `effects::windscreen_flakes_spawn`.
    pub spawn_rate: PlainFloat,
}

impl WindscreenFlakeTiming {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeTiming {
            frames: PlainDword::new(0x0),
            rate: PlainFloat::new(10.0),
            spawn_rate: PlainFloat::new(0.0),
        }
    }
}

/// The windscreen flakes' curve, circles, frame and corners (IDA 0x89CACA .. 0x89D16A).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeTables {
    /// 0x89CACA: `g_marksWindscreenFlakeCurve`: the quarter-circle curve of 0x100 floats (`effects::set_up_windscreen_flakes`; nothing ported reads it).
    pub curve: [PlainFloat; 256],
    /// 0x89CECA: `flt_89CECA` ..: the eight circles' radii the windscreen flakes' points are made on; the first also scales the axes (`effects::scale_axes`).
    pub radii: [PlainFloat; 8],
    /// 0x89CEEA: `flt_89CEEA`: 1 over the view's farthest depth (`effects::windscreen_flakes_setup`, `begin`s of `draw_pong_box`, `draw_windscreen_flake`).
    pub inverse: PlainFloat,
    /// 0x89CEEE: `flt_89CEEE`, `flt_89CEF2`: the first and second axes' lengths over the frame's divisors (`effects::windscreen_flakes_frame`, `draw_plane`).
    pub lengths: [PlainFloat; 2],
    /// 0x89CEF6: `flt_89CEF6` ..: the frame's place (the settings' +0x2720 .., as floats).
    pub place: [PlainFloat; 3],
    /// 0x89CF1A: `flt_89CF1A` ..: the frame's three axes (the first and second made from the settings, the third their cross product), each (x, y, z), made one long.
    pub axes: [[PlainFloat; 3]; 3],
    /// 0x89CF3E: `flt_89CF3E` ..: the first two axes times the first radius (`effects::scale_axes`).
    pub scaled_by_radius: [[PlainFloat; 3]; 2],
    /// 0x89CF56: `flt_89CF56` ..: the first two axes scaled by the per-part halves (`effects::scale_axes`).
    pub half_axes: [[PlainFloat; 3]; 2],
    /// 0x89CF6E: `flt_89CF6E` ..: the first two axes scaled by the lengths and the per-part factors (`effects::scale_axes`).
    pub scaled_axes: [[PlainFloat; 3]; 2],
    /// 0x89CF92: `flt_89CF92` .. `flt_89CFBE`: the four corners (x, y, depth) of the flake being drawn.
    pub corners: [[PlainFloat; 3]; 4],
    /// 0x89CFE6: `g_marksWindscreenFlakePoints`: for each of the eight radii the circle of four points (x, y, and a third float the port does not use).
    pub points: [[[PlainFloat; 3]; 4]; 8],
    /// 0x89D166: `dword_89D166`: an offset into the items, where `effects::spawn_windscreen_flake` looks for a free one.
    pub cursor: PlainDword,
}

impl WindscreenFlakeTables {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeTables {
            curve: [const { PlainFloat::new(0.0) }; 256],
            radii: [
                PlainFloat::new(1264.0),
                PlainFloat::new(4564.0),
                PlainFloat::new(4464.0),
                PlainFloat::new(4364.0),
                PlainFloat::new(4264.0),
                PlainFloat::new(4164.0),
                PlainFloat::new(4400.0),
                PlainFloat::new(4100.0),
            ],
            inverse: PlainFloat::new(0.0),
            lengths: [PlainFloat::new(37.6), PlainFloat::new(37.6)],
            place: [
                PlainFloat::new(-58982.4),
                PlainFloat::new(-29491.2),
                PlainFloat::new(145536.0),
            ],
            axes: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(1.0),
                ],
            ],
            scaled_by_radius: [
                [
                    PlainFloat::new(1664.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1188.0),
                    PlainFloat::new(1188.0),
                ],
            ],
            half_axes: [
                [
                    PlainFloat::new(1664.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(-1188.0),
                    PlainFloat::new(1188.0),
                ],
            ],
            scaled_axes: [
                [
                    PlainFloat::new(1.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                [
                    PlainFloat::new(0.0),
                    // Not 1 / sqrt 2: the game's own six digits.
                    #[allow(clippy::approx_constant)]
                    PlainFloat::new(-0.707106),
                    #[allow(clippy::approx_constant)]
                    PlainFloat::new(0.707106),
                ],
            ],
            corners: [const {
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ]
            }; 4],
            points: [const {
                [const {
                    [
                        PlainFloat::new(0.0),
                        PlainFloat::new(0.0),
                        PlainFloat::new(0.0),
                    ]
                }; 4]
            }; 8],
            cursor: PlainDword::new(0x0),
        }
    }
}

/// One windscreen flake item (0x3C bytes; `effects::spawn_windscreen_flake`, `expire_windscreen_flakes`, `run_windscreen_flakes`).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeItem {
    /// +0: 1 while the flake lives, 0 for a free item.
    pub live: PlainDword,
    /// +4: the scene clock when it was made.
    pub born: PlainDword,
    /// +8: its age in frame ticks.
    pub age: PlainDword,
    /// +0xC: the flake's texture u offset: its kind (0 .. 7) / 16 + 1/512, the tile across the sheet (set at spawn, added to the four vertices' u by `run_windscreen_flakes`).
    pub texture_u: PlainFloat,
    /// +0x10: the flake's texture v offset: 0 (from the third constant of `tile_offset_k`), added to the four vertices' v.
    pub texture_v: PlainFloat,
    /// +0x14: three dwords, always 0 (the original set them at spawn; nothing read them, so they are no longer written).
    pub spare_14: [PlainDword; 3],
    /// +0x20, +0x24: its place (x, z) as floats.
    pub place: [PlainFloat; 2],
    /// +0x28: five dwords, always 0 (the original set them at spawn; nothing read them, so they are no longer written).
    pub spare_28: [PlainDword; 5],
}

impl WindscreenFlakeItem {
    pub const fn new() -> Self {
        WindscreenFlakeItem {
            live: PlainDword::new(0),
            born: PlainDword::new(0),
            age: PlainDword::new(0),
            texture_u: PlainFloat::new(0.0),
            texture_v: PlainFloat::new(0.0),
            spare_14: [const { PlainDword::new(0) }; 3],
            place: [const { PlainFloat::new(0.0) }; 2],
            spare_28: [const { PlainDword::new(0) }; 5],
        }
    }
}

/// The windscreen flakes' 0x800 items (`g_marksWindscreenFlakeItems`) (IDA 0x89E18A .. 0x8BC18A).
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
}

/// The windscreen flakes' once-only mark, constants and randoms (IDA 0x8BC18A .. 0x8BC266).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenFlakeState {
    /// 0x8BC18A: `g_marksWindscreenFlakesDone`: 0, or where `effects::set_up_windscreen_flakes` goes on.
    pub done: PlainDword,
    /// 0x8BC18E: `flt_8BC18E`, `flt_8BC192`: the curve's two constants (1024.0, 0.0005).
    pub curve_ends: [PlainFloat; 2],
    /// 0x8BC196: `dword_8BC196`: the circles' steps (4).
    pub steps: PlainDword,
    /// 0x8BC19A: `flt_8BC19A`: the circles' angle (2.0).
    pub step_angle: PlainFloat,
    /// 0x8BC19E: `flt_8BC19E`: what the settings' rate is multiplied by (1.0; `effects::windscreen_flakes_frame`).
    pub rate_share: PlainFloat,
    /// 0x8BC1A2: `flt_8BC1A2`, `flt_8BC1A6`: the axes' lengths are divided by them (2048.0, 1024.0).
    pub length_divisors: [PlainFloat; 2],
    /// 0x8BC1AA: `flt_8BC1AA`: the rate times it must come to a whole number above 0 for the windscreen flakes to run (65536.0).
    pub share: PlainFloat,
    /// 0x8BC1AE: `dword_8BC1AE` .. `dword_8BC1BA`: the texture places of a drawn flake (u a, u c, v b, v d; `effects::draw_windscreen_flake`).
    pub places: [PlainFloat; 4],
    /// 0x8BC1BE: `flt_8BC1BE` .. `flt_8BC1CA`: 4096.0 four times (the port does not read them).
    pub unread_floats: [PlainFloat; 4],
    /// 0x8BC1CE: `dword_8BC1CE` .. `dword_8BC1DA`: 0, 0.5, 0.99999, 0.99999 (the port does not read them).
    pub unread_dwords: [PlainDword; 4],
    /// 0x8BC1DE: `flt_8BC1DE` .. `flt_8BC1F2`: the axes' per-part factors, 2048.0 three times and 1024.0 three times (`effects::scale_axes`).
    pub axis_factors: [PlainFloat; 6],
    /// 0x8BC1F6: `flt_8BC1F6` .. `flt_8BC20A`: 0.5 six times (`effects::scale_axes`).
    pub halves: [PlainFloat; 6],
    /// 0x8BC20E: `dword_8BC20E`: 256.0 (the port does not read it).
    pub unread_rhw: PlainDword,
    /// 0x8BC212: `dword_8BC212` .. `dword_8BC21E`: the texture places `effects::windscreen_flakes_setup` puts on the vertices.
    pub setup_places: [PlainFloat; 4],
    /// 0x8BC222: `dword_8BC222` .. `dword_8BC22E`: the texture places `effects::run_windscreen_flakes` puts on the vertices.
    pub run_places: [PlainFloat; 4],
    /// 0x8BC232: `dword_8BC232`: the age over which a flake fades (16).
    pub fade: PlainDword,
    /// 0x8BC236: `flt_8BC236`: 1024.0, what a flake's z is taken from (`effects::run_windscreen_flakes`).
    pub far: PlainFloat,
    /// 0x8BC23A: `flt_8BC23A`, `flt_8BC23E`, `flt_8BC242`: a new flake's texture u offset is its kind times the first (1/16) plus the second (1/512); the third (0) is its v offset (+0x10) (`effects::spawn_windscreen_flake`).
    pub tile_offset_k: [PlainFloat; 3],
    /// 0x8BC246: `flt_8BC246`, `flt_8BC24A`: 0.1 and 0 (`effects::windscreen_flakes_spawn`).
    pub rate_constants: [PlainFloat; 2],
    /// 0x8BC24E: `flt_8BC24E`: 65536.0, what the rate's fraction is multiplied by (`effects::windscreen_flakes_spawn`).
    pub share_k: PlainFloat,
    /// 0x8BC252: `dword_8BC252`: the random (times 0xC5 plus 0x5D) of whether one more flake is spawned.
    pub spawn_random: PlainDword,
    /// 0x8BC256: `dword_8BC256`: the random of a flake's place and kind (`effects::pick_windscreen_flake`).
    pub pick_random: PlainDword,
    /// 0x8BC25E: `flt_8BC25E`: the settings' +0x2750 times `EdgeSettings::factor` (`effects::scale_setting`).
    pub scaled_setting: PlainFloat,
    /// 0x8BC262: `flt_8BC262`: 50.0 and a bit (the port does not read it).
    pub unread_limit: PlainFloat,
}

impl WindscreenFlakeState {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        WindscreenFlakeState {
            done: PlainDword::new(0x0),
            curve_ends: [PlainFloat::new(1024.0), PlainFloat::new(0.00054414064)],
            steps: PlainDword::new(0x4),
            step_angle: PlainFloat::new(2.0),
            rate_share: PlainFloat::new(1.0),
            length_divisors: [PlainFloat::new(2048.0), PlainFloat::new(1024.0)],
            share: PlainFloat::new(65536.0),
            places: [
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.064453125),
                PlainFloat::new(0.064453125),
            ],
            unread_floats: [
                PlainFloat::new(16384.0),
                PlainFloat::new(16384.0),
                PlainFloat::new(16384.0),
                PlainFloat::new(16384.0),
            ],
            unread_dwords: [
                PlainDword::new(0x0),
                PlainDword::new(0x3F000000),
                PlainDword::new(0x3F7FFFFE),
                PlainDword::new(0x3F7FFFFE),
            ],
            axis_factors: [
                PlainFloat::new(2048.0),
                PlainFloat::new(2048.0),
                PlainFloat::new(2048.0),
                PlainFloat::new(1024.0),
                PlainFloat::new(1024.0),
                PlainFloat::new(1024.0),
            ],
            halves: [const { PlainFloat::new(0.5) }; 6],
            unread_rhw: PlainDword::new(0x43800000),
            setup_places: [
                PlainFloat::new(0.11328125),
                PlainFloat::new(0.03125),
                PlainFloat::new(0.24609375),
                PlainFloat::new(0.1640625),
            ],
            run_places: [
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.064453125),
                PlainFloat::new(0.064453125),
            ],
            fade: PlainDword::new(0x10),
            far: PlainFloat::new(1024.0),
            tile_offset_k: [
                PlainFloat::new(0.0625),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.0),
            ],
            rate_constants: [PlainFloat::new(0.1), PlainFloat::new(0.0)],
            share_k: PlainFloat::new(65536.0),
            spawn_random: PlainDword::new(0x0),
            pick_random: PlainDword::new(0x0),
            scaled_setting: PlainFloat::new(10.0),
            unread_limit: PlainFloat::new(50.0001),
        }
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
