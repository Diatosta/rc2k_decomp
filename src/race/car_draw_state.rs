//! A view's cars' drawing state (frame plan F4): what the drawing of the cars and of the
//! objects works out and reads again within the view, one in each view's slot
//! (`race::view_scratch`) and one for the front end.
//!
//! The original kept it in shared pieces for every view at once: `FrameWork` (the cars drawn,
//! the car's kept place and depth), `CameraWork` (the car's kept word), `RaceWork` (the car's
//! drawing word) and `FrameSwitches` (the object's material way, the fade words and the
//! number of the object being drawn), `CourseWork` and `FrameSwitches` (the car's group
//! masks) and `ViewVars` (`g_groupMask`), and each car block's +0x6FC (the state bits). The
//! old fields stay in the stores, unused.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarDrawState {
    /// Each car block's state bits as the view draws it (+0x6FC of the block in the original:
    /// lights, seen from inside 0x10, casts a shadow, ...), set at the view's start
    /// (`race::car_light_modes`), by block number.
    pub car_states: [Dword; crate::race::blocks_store::BLOCK_COUNT],
    /// The cars drawn this frame (`g_carsDrawn`).
    pub cars_drawn: Dword,
    /// The car's kept place (`car_kept` ..).
    pub car_kept: [Dword; 2],
    /// The car's depth (`car_depth`, 0x50000 at first).
    pub car_depth: Dword,
    /// The car's kept word (0x2100000 while a car is drawn).
    pub car_draw_kept: Dword,
    /// The car's drawing word (0x60 plus its nearness size).
    pub car_draw_word: Dword,
    /// The drawn car's nearness, 0 .. 1 (`car_nearness`).
    pub car_nearness: crate::session::Float,
    /// The drawn car's mode (`g_groupsWhich`, 0x6D18D4: its block's `mode_on`), read by the
    /// face tests (`scene::list_object_face`).
    pub groups_which: Dword,
    /// The drawn car's four parts' matrices (`FrameWork.part_matrices`, 0x6417E0 ..: copied
    /// from its block's +0x740 by `race::draw_car`).
    pub part_matrices: [crate::session::Float; 64],
    /// The drawn car's turn (`FrameWork.placed_matrices`' first, 0x641760: its block's
    /// +0x880).
    pub car_turn: [crate::session::Float; 16],
    /// The material way of the object being drawn (`other_way_kind`).
    pub other_way_kind: Dword,
    /// The other kind and the distance of the object's fade (`fade_words`).
    pub fade_words: [Dword; 2],
    /// The object being drawn or set up (`g_objectNumber`).
    pub object_number: Dword,
    /// The drawn car's model's group masks (`COURSE.group_masks`, 0x5CCE1C ..: its parts'
    /// groups by slot, set by `race::draw_car` for each car, [13] .. [17] read by
    /// `scene::list_object_face`; outside a race [13], [14] the known kinds' hidden groups).
    pub group_masks: [Dword; 18],
    /// The groups of the car's parts still on it (`group_mask`).
    pub shown_groups: Dword,
    /// The face group mask of the car being drawn (`g_groupMask`, 0x606280; -1 after a car).
    pub group_mask: Dword,
    /// The car's parts present (`FRAME_WORK.parts_present`, 0x641754), each view's own.
    pub parts_present: Dword,
    /// The left light's byte (`FRAME_WORK.light_left`, 0x641758), each view's own.
    pub light_left: Dword,
    /// The right light's byte (`FRAME_WORK.light_right`, 0x64175C), each view's own.
    pub light_right: Dword,
}

impl CarDrawState {
    pub const fn new() -> Self {
        Self {
            car_states: [const { Dword::new(0) }; crate::race::blocks_store::BLOCK_COUNT],
            cars_drawn: Dword::new(0),
            car_kept: [const { Dword::new(0) }; 2],
            car_depth: Dword::new(0),
            car_draw_kept: Dword::new(0),
            car_draw_word: Dword::new(0),
            car_nearness: crate::session::Float::from_bits(0),
            groups_which: Dword::new(0),
            part_matrices: [const { crate::session::Float::from_bits(0) }; 64],
            car_turn: [const { crate::session::Float::from_bits(0) }; 16],
            other_way_kind: Dword::new(0),
            fade_words: [const { Dword::new(0) }; 2],
            object_number: Dword::new(0),
            group_masks: [const { Dword::new(0) }; 18],
            shown_groups: Dword::new(0),
            group_mask: Dword::new(0),
            parts_present: Dword::new(0),
            light_left: Dword::new(0),
            light_right: Dword::new(0),
        }
    }
}

impl CarDrawState {
    /// The state bits of the car whose block is at `block`.
    pub fn state_of(&self, block: u32) -> Result<&Dword, crate::errors::BadInput> {
        crate::physics::car_store::Car::block_number(block)
            .map(|n| &self.car_states[n])
            .ok_or(crate::errors::BadInput(
                "a car's state bits not of a car block",
            ))
    }
}
