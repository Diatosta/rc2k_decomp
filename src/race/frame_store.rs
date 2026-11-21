//! The frame's draw work: the sun's depth and size, the mirror, the views' layouts and scales, the
//! lead car's camera kept, each car's listed sound, and the car parts' matrices and decal places.

use crate::memory::session::{ConstFloat, DrawPlain, PlainRead};
use crate::scene::matrix::Matrix;

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

/// The frame's draw work as a plain record (`session::DrawPlain`): written on the game's thread
/// between frames and read by the drawing parts as a copy made when a drawing scope opens.
#[derive(Clone, Copy)]
pub struct FrameWork {
    /// The car's depth, 0x50000 at first. Unused: each view keeps its own
    /// (`ctx.view.cars.car_depth`).
    pub car_depth: u32,
    /// Whether the sun is on the screen, set by the polygon hit test. Unused: each view keeps its
    /// own (`ViewSun`).
    pub sun_on_screen: u32,
    /// The sun's point on the screen, x and y. Unused: each view keeps its own (`ViewSun::hit`).
    pub sun_hit: [u32; 2],
    /// The view's value read by the listener's placing. Unused: each view keeps its own
    /// (`ViewSun::view_out`).
    pub view_out: u32,
    /// The sun's size. Unused: each view keeps its own (`ViewSun::size`).
    pub sun_size: u32,
    /// Whether the mirror's picture is wanted this frame.
    pub mirror_wanted: bool,
    /// The mirror's switch, used by `race::frame_views`.
    pub mirror_switch: u32,
    /// The views' values kept between steps of `race::frame_views`.
    pub view_kept: [u32; 2],
    /// The view textures to load again, one bit each.
    pub view_textures_forgotten: u32,
    /// Set when the kinds' textures are done for the frame.
    pub kind_textures_done: u32,
    /// The lead car's camera values kept (`drive::lead_car_values`); [0] is also
    /// `effects::stage_record`'s number, [7] `effects::draw_plane`'s switch and `effects::pong`'s
    /// shown flag.
    pub lead_kept: [u32; 8],
    /// Each car's listed sound, -1 for none.
    pub listed_car_sounds: [u32; 8],
    /// Set while a view is being drawn.
    pub drawing: u32,
    /// 1 while the replay's still frame is shown (set and cleared by `race::show_replay_still`);
    /// `race::lay_skidmarks_if_on` lays no skid marks while it is set.
    pub replay_still_on: u32,
    /// The car's place for the camera.
    pub camera_car: [u32; 3],
    /// The car part's place used by `race::frame_draw`.
    pub part_place: [u32; 3],
    /// The car's placed matrix, used by `race::car_parts`; the drawing's is the view's
    /// (`CarDrawState::car_turn`).
    pub placed_matrix: Matrix,
    /// The car parts' four matrices, used by `race::car_parts`; the drawing's are the view's
    /// (`CarDrawState::part_matrices`).
    pub part_matrices: [Matrix; 4],
}

impl FrameWork {
    /// The work with its first values.
    pub const fn new() -> Self {
        Self {
            car_depth: 0,
            sun_on_screen: 0,
            sun_hit: [0; 2],
            view_out: 0,
            sun_size: 0,
            mirror_wanted: false,
            mirror_switch: 0,
            view_kept: [0; 2],
            view_textures_forgotten: 0,
            kind_textures_done: 0,
            lead_kept: [0; 8],
            listed_car_sounds: [0; 8],
            drawing: 0,
            replay_still_on: 0,
            camera_car: [0; 3],
            part_place: [0xFFFFA000, 0xC000, 0x20000],
            placed_matrix: [0.0; 16],
            part_matrices: [[0.0; 16]; 4],
        }
    }
}

/// The work as one reader finds it: the game's thread's own (read and written through the store,
/// short borrows), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum FrameWorkRef<'a> {
    Store(&'a DrawPlain<FrameWork>),
    Copy(&'a FrameWork),
}

impl FrameWorkRef<'_> {
    /// Each car's listed sound.
    pub fn listed_car_sounds(self) -> [u32; 8] {
        match self {
            Self::Store(store) => store.borrow().listed_car_sounds,
            Self::Copy(record) => record.listed_car_sounds,
        }
    }

    /// Whether a view is being drawn.
    pub fn drawing(self) -> u32 {
        match self {
            Self::Store(store) => store.borrow().drawing,
            Self::Copy(record) => record.drawing,
        }
    }

    /// The car part's place.
    pub fn part_place(self) -> [u32; 3] {
        match self {
            Self::Store(store) => store.borrow().part_place,
            Self::Copy(record) => record.part_place,
        }
    }
}

impl<'a> From<&'a DrawPlain<FrameWork>> for FrameWorkRef<'a> {
    fn from(store: &'a DrawPlain<FrameWork>) -> Self {
        Self::Store(store)
    }
}

impl<'a> From<&'a PlainRead<FrameWork>> for FrameWorkRef<'a> {
    fn from(copy: &'a PlainRead<FrameWork>) -> Self {
        Self::Copy(copy)
    }
}

/// The work as the game's thread holds it.
pub type FrameWorkStore = DrawPlain<FrameWork>;
