//! The frame's draw work: the counts of a frame (flares, lamps, cars drawn and culled), the sun's
//! place and size, the mirror, the views' layouts and scales, the lead car's camera kept, each
//! car's listed sound, the car parts' matrices and decal places, and the car shadow's work. The
//! fields keep a fixed layout.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFlag, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameWork {
    /// The flares this frame. Unused: each view keeps its own (`race::lamp_work::LampWork`).
    pub flare_count: PlainDword,
    /// Unused; nothing reads or writes it.
    pub lamp_entry_count: PlainDword,
    /// Unused: each view counts its own (`ctx.view.cars.cars_drawn`).
    pub cars_drawn: PlainDword,
    /// Unused; nothing reads or writes it.
    pub cars_culled: PlainDword,
    /// Unused: each view keeps its own (`ctx.view.cars.car_kept`).
    pub car_kept: [PlainDword; 2],
    /// The car's depth, 0x50000 at first. Unused: each view keeps its own
    /// (`ctx.view.cars.car_depth`).
    pub car_depth: PlainDword,
    /// The sun's depth, used by `race::frame_draw`.
    pub sun_depth: PlainFloat,
    /// The sun's direction; nothing reads or writes it now.
    pub sun_floats: [PlainFloat; 3],
    /// The sun's turned point; nothing reads or writes it now.
    pub sun_points: [PlainDword; 3],
    /// Whether the sun is on the screen, set by the polygon hit test. Unused: each view keeps its
    /// own (`ViewSun`).
    pub sun_on_screen: PlainDword,
    /// The sun's point on the screen, x and y. Unused: each view keeps its own (`ViewSun::hit`).
    pub sun_hit: [PlainDword; 2],
    /// The sun's x and y as float bits. Unused: each view keeps its own (`ViewSun::place`).
    pub sun_place: [PlainDword; 2],
    /// The view's value read by the listener's placing. Unused: each view keeps its own
    /// (`ViewSun::view_out`).
    pub view_out: PlainDword,
    /// The sun's size. Unused: each view keeps its own (`ViewSun::size`).
    pub sun_size: PlainDword,
    /// Whether the mirror's picture is wanted this frame.
    pub mirror_wanted: PlainFlag,
    /// Unused: each view keeps its own (`scene::object_light`).
    pub view_light: PlainDword,
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
    /// Unused: each view keeps its own (`race::face_work::FaceWork`).
    pub face_hide_mask: PlainDword,
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
    /// Unused; only keeps the layout.
    pub unused_168: PlainDword,
    /// The car's decal places.
    pub car_decal_places: [PlainDword; 18],
    /// The decal limit used by `race::car_draw`.
    pub decal_limit: PlainFloat,
    /// The car part's place used by `race::frame_draw`.
    pub part_place: [PlainDword; 3],
    /// Written by nothing now; unused.
    pub part_kind: PlainDword,
    /// The car's parts present. Unused: each view keeps its own
    /// (`race::car_draw_state::CarDrawState`).
    pub parts_present: PlainDword,
    /// Unused: each view keeps its own (`race::car_draw_state::CarDrawState`).
    pub light_left: PlainDword,
    /// Unused: each view keeps its own (`race::car_draw_state::CarDrawState`).
    pub light_right: PlainDword,
    /// The car's placed matrix and the next, used by `race::car_parts`; the drawing's are the
    /// view's (`CarDrawState::car_turn`).
    pub placed_matrices: [PlainFloat; 32],
    /// The car parts' four matrices, used by `race::car_parts`; the drawing's are the view's
    /// (`CarDrawState::part_matrices`).
    pub part_matrices: [PlainFloat; 64],
    /// Unused; only keeps the layout.
    pub unused_354: [PlainDword; 18],
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_strength: PlainDword,
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_point_count: PlainDword,
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_texture: PlainDword,
    /// Unused; only keeps the layout.
    pub unused_3a8: PlainDword,
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_light: [PlainDword; 3],
    /// Unused; only keeps the layout.
    pub unused_3b8: PlainDword,
    /// The shadow's corners, x and z twice. Unused: the shadow keeps its own work
    /// (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_corners: [PlainDword; 4],
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_work: [PlainDword; 14],
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_scene: PlainDword,
    /// The shadow circle's radius and unit, 2^23 and 2^24; never written
    /// (`race::car_shadow::CIRCLE`).
    pub shadow_circle: [PlainFloat; 2],
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_list: [PlainDword; 64],
    /// Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_list_count: PlainDword,
    /// The ground's height. Unused: the shadow keeps its own work (`race::car_shadow::ShadowWork`);
    /// kept for the layout.
    pub shadow_ground: PlainDword,
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
            shadow_circle: [PlainFloat::new(8388608.0), PlainFloat::new(16777216.0)],
            ..ZeroCells::ZERO
        }
    }
}
