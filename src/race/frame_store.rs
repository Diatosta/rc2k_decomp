//! The frame's draw work: the sun's depth and size, the mirror, the views' layouts and scales, the
//! lead car's camera kept, each car's listed sound, and the car parts' matrices and decal places.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFlag, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameWork {
    /// The car's depth, 0x50000 at first. Unused: each view keeps its own
    /// (`ctx.view.cars.car_depth`).
    pub car_depth: PlainDword,
    /// The sun's depth, used by `race::frame_draw`.
    pub sun_depth: PlainFloat,
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
    /// 12 entries of two dwords, picked by `race::mirror_entry`.
    pub mirror_places: [PlainDword; 24],
    /// The lead car's camera values kept (`drive::lead_car_values`); [0] is also
    /// `effects::stage_record`'s number, [7] `effects::draw_plane`'s switch and `effects::pong`'s
    /// shown flag.
    pub lead_kept: [PlainDword; 8],
    /// Each car's listed sound, -1 for none.
    pub listed_car_sounds: [PlainDword; 8],
    /// The views' layouts, used by `race::frame_views`.
    pub view_layouts: [PlainDword; 6],
    /// Set while a view is being drawn.
    pub drawing: PlainDword,
    /// 1 while the replay's still frame is shown (set and cleared by `race::show_replay_still`);
    /// `race::lay_skidmarks_if_on` lays no skid marks while it is set.
    pub replay_still_on: PlainDword,
    /// The views' horizontal scales, by layout.
    pub view_scale_x: [PlainFloat; 5],
    /// The views' vertical scales, by layout.
    pub view_scale_y: [PlainFloat; 5],
    /// The car's place for the camera.
    pub camera_car: [PlainDword; 3],
    /// The car's decal places.
    pub car_decal_places: [PlainDword; 18],
    /// The decal limit used by `race::car_draw`.
    pub decal_limit: PlainFloat,
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
            sun_depth: PlainFloat::new(0.024543693),
            mirror_places: [
                PlainDword::new(0x25D),
                PlainDword::new(0x30),
                PlainDword::new(0xD6),
                PlainDword::new(0x24),
                PlainDword::new(0x25D),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(0x30),
                PlainDword::new(0x268),
                PlainDword::new(0x24),
                PlainDword::new(0x25D),
                PlainDword::new(0x141),
                PlainDword::new(0x25D),
                PlainDword::new(0x10),
                PlainDword::new(0xD6),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(0x10),
                PlainDword::new(0x268),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(0x131),
            ],
            view_layouts: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(3),
                PlainDword::new(4),
                PlainDword::new(1),
                PlainDword::new(2),
            ],
            view_scale_x: [
                PlainFloat::new(400.0),
                PlainFloat::new(400.0),
                PlainFloat::new(400.0),
                PlainFloat::new(200.0),
                PlainFloat::new(600.0),
            ],
            view_scale_y: [
                PlainFloat::new(300.0),
                PlainFloat::new(150.0),
                PlainFloat::new(450.0),
                PlainFloat::new(300.0),
                PlainFloat::new(300.0),
            ],
            car_decal_places: [
                PlainDword::new(0xA),
                PlainDword::new(0xB),
                PlainDword::new(0x18),
                PlainDword::new(0x19),
                PlainDword::new(0xC),
                PlainDword::new(0xD),
                PlainDword::new(0x16),
                PlainDword::new(0x17),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0xE),
                PlainDword::new(0xF),
                PlainDword::new(0x10),
                PlainDword::new(0x11),
                PlainDword::new(0x13),
                PlainDword::new(0x14),
                PlainDword::new(0x12),
                PlainDword::new(0),
            ],
            decal_limit: PlainFloat::new(5.0),
            part_place: [
                PlainDword::new(0xFFFFA000),
                PlainDword::new(0xC000),
                PlainDword::new(0x20000),
            ],
            ..ZeroCells::ZERO
        }
    }
}
