//! The windscreen flakes' variables: their frame count and rates (`WindscreenFlakeTiming`), curve,
//! circles and frame (`WindscreenFlakeTables`), the 0x800 items (`WindscreenFlakeItems`), and the
//! once-only mark, constants and randoms (`WindscreenFlakeState`).

/// The windscreen flakes' frame count and rates.
pub struct WindscreenFlakeTiming {
    /// The frames the windscreen flakes have run (`effects::windscreen_flakes_spawn` adds one each
    /// call).
    pub frames: u32,
    /// The windscreen flakes' rate this frame, the settings' `windscreen_flakes_rate_factor` times
    /// `WindscreenFlakeState::rate_share` (`effects::windscreen_flakes_frame`, `draw_plane`); 10.0
    /// at first.
    pub rate: f32,
    /// The windscreen flakes spawned a frame, worked out from the rate by
    /// `effects::windscreen_flakes_spawn`.
    pub spawn_rate: f32,
}

impl WindscreenFlakeTiming {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WindscreenFlakeTiming {
            frames: 0,
            rate: 10.0,
            spawn_rate: 0.0,
        }
    }
}

/// The windscreen flakes' curve, circles, frame and corners.
pub struct WindscreenFlakeTables {
    /// The quarter-circle curve of 0x100 floats (`effects::set_up_windscreen_flakes`; nothing
    /// reads it).
    pub curve: [f32; 256],
    /// The eight circles' radii the windscreen flakes' points are made on; the first also scales
    /// the axes (`effects::scale_axes`).
    pub radii: [f32; 8],
    /// 1 over the view's farthest depth (`effects::windscreen_flakes_setup`, `begin`s of
    /// `draw_pong_box`, `draw_windscreen_flake`).
    pub inverse: f32,
    /// The first and second axes' lengths over the frame's divisors
    /// (`effects::windscreen_flakes_frame`, `draw_plane`).
    pub lengths: [f32; 2],
    /// The frame's place (the settings' `light_point_b_negated`, as floats).
    pub place: [f32; 3],
    /// The frame's three axes (the first and second made from the settings, the third their cross
    /// product), each (x, y, z), made one long.
    pub axes: [[f32; 3]; 3],
    /// The first two axes times the first radius (`effects::scale_axes`).
    pub scaled_by_radius: [[f32; 3]; 2],
    /// The first two axes scaled by the per-part halves (`effects::scale_axes`).
    pub half_axes: [[f32; 3]; 2],
    /// The first two axes scaled by the lengths and the per-part factors (`effects::scale_axes`).
    pub scaled_axes: [[f32; 3]; 2],
    /// The four corners (x, y, depth) of the flake being drawn.
    pub corners: [[f32; 3]; 4],
    /// For each of the eight radii the circle of four points (x, y, and a third float that
    /// is not used).
    pub points: [[[f32; 3]; 4]; 8],
    /// An offset into the items, where `effects::spawn_windscreen_flake` looks for a free one.
    pub cursor: u32,
}

impl WindscreenFlakeTables {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WindscreenFlakeTables {
            curve: [0.0; 256],
            radii: [
                1264.0, 4564.0, 4464.0, 4364.0, 4264.0, 4164.0, 4400.0, 4100.0,
            ],
            inverse: 0.0,
            lengths: [37.6, 37.6],
            place: [-58982.4, -29491.2, 145536.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            scaled_by_radius: [[1664.0, 0.0, 0.0], [0.0, -1188.0, 1188.0]],
            half_axes: [[1664.0, 0.0, 0.0], [0.0, -1188.0, 1188.0]],
            scaled_axes: [
                [1.0, 0.0, 0.0],
                [
                    0.0,
                    // Not 1 / sqrt 2: the game's own six digits.
                    #[allow(clippy::approx_constant)]
                    -0.707106,
                    #[allow(clippy::approx_constant)]
                    0.707106,
                ],
            ],
            corners: [[0.0, 0.0, 0.0]; 4],
            points: [const { [[0.0, 0.0, 0.0]; 4] }; 8],
            cursor: 0x0,
        }
    }
}

/// One windscreen flake item (`effects::spawn_windscreen_flake`, `expire_windscreen_flakes`,
/// `run_windscreen_flakes`).
#[derive(Clone, Copy)]
pub struct WindscreenFlakeItem {
    /// 1 while the flake lives, 0 for a free item.
    pub live: u32,
    /// The scene clock when it was made.
    pub born: u32,
    /// Its age in frame ticks.
    pub age: u32,
    /// The flake's texture u offset: its kind (0 .. 7) / 16 + 1/512, the tile across the
    /// sheet (set at spawn, added to the four vertices' u by `run_windscreen_flakes`).
    pub texture_u: f32,
    /// The flake's texture v offset: 0 (from the third constant of `tile_offset_k`), added
    /// to the four vertices' v.
    pub texture_v: f32,
    /// Its place (x, z) as floats.
    pub place: [f32; 2],
}

impl WindscreenFlakeItem {
    pub const fn new() -> Self {
        WindscreenFlakeItem {
            live: 0,
            born: 0,
            age: 0,
            texture_u: 0.0,
            texture_v: 0.0,
            place: [0.0; 2],
        }
    }
}

/// The windscreen flakes' 0x800 items.
pub struct WindscreenFlakeItems {
    /// The items.
    pub items: [WindscreenFlakeItem; 0x800],
}

impl WindscreenFlakeItems {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WindscreenFlakeItems {
            items: [const { WindscreenFlakeItem::new() }; 0x800],
        }
    }
}

/// The windscreen flakes' once-only mark, constants and randoms.
pub struct WindscreenFlakeState {
    /// No code reads it (it was a once-only mark with nothing done once).
    pub unused_done: u32,
    /// The curve's two constants (1024.0, 0.0005).
    pub curve_ends: [f32; 2],
    /// The circles' steps (4).
    pub steps: u32,
    /// The circles' angle (2.0).
    pub step_angle: f32,
    /// What the settings' rate is multiplied by (1.0; `effects::windscreen_flakes_frame`).
    pub rate_share: f32,
    /// The axes' lengths are divided by them (2048.0, 1024.0).
    pub length_divisors: [f32; 2],
    /// The rate times it must come to a whole number above 0 for the windscreen flakes to run
    /// (65536.0).
    pub share: f32,
    /// The texture places of a drawn flake (u a, u c, v b, v d; `effects::draw_windscreen_flake`).
    pub places: [f32; 4],
    /// 4096.0 four times (nothing reads them).
    pub unread_floats: [f32; 4],
    /// 0, 0.5, 0.99999, 0.99999 (nothing reads them).
    pub unread_dwords: [u32; 4],
    /// The axes' per-part factors, 2048.0 three times and 1024.0 three times
    /// (`effects::scale_axes`).
    pub axis_factors: [f32; 6],
    /// 0.5 six times (`effects::scale_axes`).
    pub halves: [f32; 6],
    /// 256.0 (nothing reads it).
    pub unread_rhw: u32,
    /// The texture places `effects::windscreen_flakes_setup` puts on the vertices.
    pub setup_places: [f32; 4],
    /// The texture places `effects::run_windscreen_flakes` puts on the vertices.
    pub run_places: [f32; 4],
    /// The age over which a flake fades (16).
    pub fade: u32,
    /// 1024.0, what a flake's z is taken from (`effects::run_windscreen_flakes`).
    pub far: f32,
    /// A new flake's texture u offset is its kind times the first (1/16) plus the second (1/512);
    /// the third (0) is its v offset (`texture_v`) (`effects::spawn_windscreen_flake`).
    pub tile_offset_k: [f32; 3],
    /// 0.1 and 0 (`effects::windscreen_flakes_spawn`).
    pub rate_constants: [f32; 2],
    /// 65536.0, what the rate's fraction is multiplied by (`effects::windscreen_flakes_spawn`).
    pub share_k: f32,
    /// The random (times 0xC5 plus 0x5D) of whether one more flake is spawned.
    pub spawn_random: u32,
    /// The random of a flake's place and kind (`effects::pick_windscreen_flake`).
    pub pick_random: u32,
    /// The settings' `setting_to_scale` times `EdgeSettings::factor` (`effects::scale_setting`).
    pub scaled_setting: f32,
    /// 50.0 and a bit (nothing reads it).
    pub unread_limit: f32,
}

impl WindscreenFlakeState {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WindscreenFlakeState {
            unused_done: 0x0,
            curve_ends: [1024.0, 0.00054414064],
            steps: 0x4,
            step_angle: 2.0,
            rate_share: 1.0,
            length_divisors: [2048.0, 1024.0],
            share: 65536.0,
            places: [0.001953125, 0.001953125, 0.064453125, 0.064453125],
            unread_floats: [16384.0, 16384.0, 16384.0, 16384.0],
            unread_dwords: [0x0, 0x3F000000, 0x3F7FFFFE, 0x3F7FFFFE],
            axis_factors: [2048.0, 2048.0, 2048.0, 1024.0, 1024.0, 1024.0],
            halves: [0.5; 6],
            unread_rhw: 0x43800000,
            setup_places: [0.11328125, 0.03125, 0.24609375, 0.1640625],
            run_places: [0.001953125, 0.001953125, 0.064453125, 0.064453125],
            fade: 0x10,
            far: 1024.0,
            tile_offset_k: [0.0625, 0.001953125, 0.0],
            rate_constants: [0.1, 0.0],
            share_k: 65536.0,
            spawn_random: 0x0,
            pick_random: 0x0,
            scaled_setting: 10.0,
            unread_limit: 50.0001,
        }
    }
}

/// The windscreen flakes' state, one part of the effects' owner (`effects::owner::Effects`).
/// The pong box and the plane draw with its frame and corners too.
pub struct WindscreenFlakes {
    pub timing: std::cell::RefCell<WindscreenFlakeTiming>,
    pub tables: std::cell::RefCell<WindscreenFlakeTables>,
    pub items: std::cell::RefCell<WindscreenFlakeItems>,
    pub state: std::cell::RefCell<WindscreenFlakeState>,
}

impl WindscreenFlakes {
    pub const fn new() -> Self {
        WindscreenFlakes {
            timing: std::cell::RefCell::new(WindscreenFlakeTiming::new()),
            tables: std::cell::RefCell::new(WindscreenFlakeTables::new()),
            items: std::cell::RefCell::new(WindscreenFlakeItems::new()),
            state: std::cell::RefCell::new(WindscreenFlakeState::new()),
        }
    }
}
