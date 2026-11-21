//! The frame's draw work: the sun's depth and size, the mirror, the views' layouts and scales, the
//! lead car's camera kept, each car's listed sound, and the car parts' matrices and decal places.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{ConstFloat, PlainDword, PlainFlag, PlainFloat};

/// The sun's depth, used by `race::frame_draw`.
pub const SUN_DEPTH: ConstFloat = ConstFloat::new(0.024543693);

/// The decal limit used by `race::car_draw`.
pub const DECAL_LIMIT: ConstFloat = ConstFloat::new(5.0);

/// 12 entries of two dwords, picked by `race::mirror_entry`.
pub const MIRROR_PLACES: [u32; 24] = [
    0x25D, 0x30, 0xD6, 0x24, 0x25D, 4, 0x25D, 0x30, 0x268, 0x24, 0x25D, 0x141, 0x25D, 0x10, 0xD6,
    4, 0x25D, 4, 0x25D, 0x10, 0x268, 4, 0x25D, 0x131,
];

/// The views' layouts, used by `race::frame_views`.
pub const VIEW_LAYOUTS: [u32; 6] = [0, 0, 3, 4, 1, 2];

/// The views' horizontal scales, by layout.
pub const VIEW_SCALE_X: [f32; 5] = [400.0, 400.0, 400.0, 200.0, 600.0];

/// The views' vertical scales, by layout.
pub const VIEW_SCALE_Y: [f32; 5] = [300.0, 150.0, 450.0, 300.0, 300.0];

/// The car's decal places.
pub const CAR_DECAL_PLACES: [u32; 18] = [
    0xA, 0xB, 0x18, 0x19, 0xC, 0xD, 0x16, 0x17, 0x1A, 0x1B, 0xE, 0xF, 0x10, 0x11, 0x13, 0x14, 0x12,
    0,
];

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameWork {
    /// The car's depth, 0x50000 at first. Unused: each view keeps its own
    /// (`ctx.view.cars.car_depth`).
    pub car_depth: PlainDword,
    /// Whether the sun is on the screen, set by the polygon hit test. Unused: each view keeps its
    /// own (`ViewSun`).
    pub sun_on_screen: PlainDword,
    /// The sun's point on the screen, x and y. Unused: each view keeps its own (`ViewSun::hit`).
    pub sun_hit: [PlainDword; 2],
    /// The view's value read by the listener's placing. Unused: each view keeps its own
    /// (`ViewSun::view_out`).
    pub view_out: PlainDword,
    /// The sun's size. Unused: each view keeps its own (`ViewSun::size`).
    pub sun_size: PlainDword,
    /// Whether the mirror's picture is wanted this frame.
    pub mirror_wanted: PlainFlag,
    /// The mirror's switch, used by `race::frame_views`.
    pub mirror_switch: PlainDword,
    /// The views' values kept between steps of `race::frame_views`.
    pub view_kept: [PlainDword; 2],
    /// The view textures to load again, one bit each.
    pub view_textures_forgotten: PlainDword,
    /// Set when the kinds' textures are done for the frame.
    pub kind_textures_done: PlainDword,
    /// The lead car's camera values kept (`drive::lead_car_values`); [0] is also
    /// `effects::stage_record`'s number, [7] `effects::draw_plane`'s switch and `effects::pong`'s
    /// shown flag.
    pub lead_kept: [PlainDword; 8],
    /// Each car's listed sound, -1 for none.
    pub listed_car_sounds: [PlainDword; 8],
    /// Set while a view is being drawn.
    pub drawing: PlainDword,
    /// 1 while the replay's still frame is shown (set and cleared by `race::show_replay_still`);
    /// `race::lay_skidmarks_if_on` lays no skid marks while it is set.
    pub replay_still_on: PlainDword,
    /// The car's place for the camera.
    pub camera_car: [PlainDword; 3],
    /// The car part's place used by `race::frame_draw`.
    pub part_place: [PlainDword; 3],
    /// The car's placed matrix and the next, used by `race::car_parts`; the drawing's are the
    /// view's (`CarDrawState::car_turn`).
    pub placed_matrices: [PlainFloat; 32],
    /// The car parts' four matrices, used by `race::car_parts`; the drawing's are the view's
    /// (`CarDrawState::part_matrices`).
    pub part_matrices: [PlainFloat; 64],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The work with its first values.
impl FrameWork {
    pub const fn new() -> Self {
        Self {
            part_place: [
                PlainDword::new(0xFFFFA000),
                PlainDword::new(0xC000),
                PlainDword::new(0x20000),
            ],
            ..ZeroCells::ZERO
        }
    }
}
