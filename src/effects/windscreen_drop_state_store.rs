//! The windscreen drops' and windscreen flakes' variables: `WindscreenDropState` and
//! `WindscreenDropWork`.

/// The windscreen drops' and windscreen flakes' frame variables (`effects::windscreen_drops_*`,
/// `effects::windscreen_flakes_*`, `effects::draw_plane`).
pub struct WindscreenDropState {
    /// The live windscreen drop items counted in this frame (`effects::windscreen_drops_frame`).
    pub live: u32,
    /// The clock the items' steps are counted from (`effects::windscreen_drops_step`,
    /// `effects::windscreen_flakes_step`).
    pub clock: u32,
    /// The place's way dotted with the third axis, negative when it faces the third axis away
    /// (`effects::windscreen_drops_top`, `effects::windscreen_flakes_frame`,
    /// `effects::draw_plane`).
    pub facing: f32,
    /// The cell (x, z) of the item being looked at (`effects::expire_windscreen_drops`,
    /// `effects::expire_windscreen_flakes`).
    pub cell: [u32; 2],
    /// The frames counted (`effects::windscreen_drops_spawn`).
    pub frames: u32,
    /// The windscreen drops' rate (`effects::windscreen_drops_top`,
    /// `effects::windscreen_drops_spawn`).
    pub rate: f32,
    /// The spawn rate worked out each frame (`effects::windscreen_drops_spawn`).
    pub spawn_rate: f32,
    /// The sprite's depth (`effects::windscreen_drops_frame`).
    pub depth: f32,
}

impl WindscreenDropState {
    pub const fn new() -> Self {
        WindscreenDropState {
            live: 0x0,
            clock: 0x0,
            facing: 0.0,
            cell: [0x0, 0x0],
            frames: 0x0,
            rate: 0.0,
            spawn_rate: 1.125,
            depth: 0.0,
        }
    }
}

/// The windscreen drops' work (`effects::windscreen_drops_*`, `effects::shapes`,
/// `effects::spawn_windscreen_drop`): the drift, the curve, the radii, the axes, the sprite's
/// corners and the windscreen drop points.
pub struct WindscreenDropWork {
    /// The drift at the item's cell (`effects::windscreen_drops_frame`).
    pub drift: [f32; 2],
    /// A quarter-circle curve of 0x100 floats (`effects::shapes`).
    pub curve: [f32; 256],
    /// The circles' radii (eight kinds used).
    pub radii: [f32; 16],
    /// 1 over the view's farthest depth (`effects::windscreen_drops_setup`).
    pub inverse: f32,
    /// The axes' lengths' shares (`effects::windscreen_drops_top`).
    pub shares: [f32; 2],
    /// The view's place (the settings' `light_point_b_negated`).
    pub place: [f32; 3],
    /// The three axes (rows of three).
    pub axes: [f32; 9],
    /// The two axes scaled (`effects::windscreen_drops_setup`).
    pub setup_axes: [f32; 12],
    /// The sprite's centre on the screen (`effects::windscreen_drops_frame`).
    pub screen: [f32; 2],
    /// The item's place turned, its depth last.
    pub cell: [f32; 3],
    /// The six corners (x, y, z).
    pub corners: [f32; 18],
    /// The 0x30 points scaled and turned onto the axes (`effects::windscreen_drops_top`).
    pub scaled: [f32; 144],
    /// 0x30 points of 12 bytes (eight circles of six).
    pub points: [f32; 144],
    /// An offset into the items (`effects::spawn_windscreen_drop`).
    pub cursor: u32,
    /// The item's place as whole numbers (x, z).
    pub whole: [u32; 2],
}

impl WindscreenDropWork {
    pub const fn new() -> Self {
        WindscreenDropWork {
            drift: [0.0, 0.0],
            curve: [0.0; 256],
            radii: [
                7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 4.0, 5.0, 2.0, 1.8, 1.6, 1.4, 1.2, 1.0, 1.2, 1.4,
            ],
            inverse: 0.0,
            shares: [57.6, 57.6],
            place: [-58982.4, -29491.2, 65536.0],
            axes: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            setup_axes: [
                1664.0, 0.0, 0.0, 0.0, -1188.0, 1188.0, 1664.0, 0.0, 0.0, 0.0, -1188.0, 1188.0,
            ],
            screen: [0.0, 0.0],
            cell: [0.0, 0.0, 0.0],
            corners: [0.0; 18],
            scaled: [0.0; 144],
            points: [0.0; 144],
            cursor: 0x0,
            whole: [0x0, 0x0],
        }
    }
}
