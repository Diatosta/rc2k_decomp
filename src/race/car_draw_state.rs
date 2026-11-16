//! A view's cars' drawing state: what drawing the cars and the objects works out and reads again
//! within the view, one in each view's slot (`view_scratch`) and one for the front end. It replaces
//! state once shared by every view at once: `FrameWork` (the cars drawn, the car's kept place and
//! depth), `CameraWork` (the car's kept word), `RaceWork` (the car's drawing word), `FrameSwitches`
//! (the object's material way, the fade words, the number of the object being drawn, the car's
//! group masks), `CourseWork` and `ViewVars` (`group_mask`), and each car block's state bits
//! (+0x6FC). The old fields stay in the stores, unused.

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells)]
pub struct CarDrawState {
    /// Each car block's state bits as the view draws it (+0x6FC of the block: lights, seen from
    /// inside 0x10, casts a shadow, ...), set at the view's start (`car_light_modes`), by block
    /// number.
    pub car_states: [PlainDword; crate::race::blocks_store::BLOCK_COUNT],
    /// The cars drawn this frame (`cars_drawn`).
    pub cars_drawn: PlainDword,
    /// The car's kept place (`car_kept` ..).
    pub car_kept: [PlainDword; 2],
    /// The car's depth (`car_depth`, 0x50000 at first).
    pub car_depth: PlainDword,
    /// The car's kept word (0x2100000 while a car is drawn).
    pub car_draw_kept: PlainDword,
    /// The car's drawing word (0x60 plus its nearness size).
    pub car_draw_word: PlainDword,
    /// The drawn car's nearness, 0 .. 1 (`car_nearness`).
    pub car_nearness: crate::memory::session::PlainFloat,
    /// The drawn car's mode (`groups_which`: its block's `mode_on`), read by the face tests
    /// (`list_object_face`).
    pub groups_which: PlainDword,
    /// The drawn car's four parts' matrices (`FrameWork.part_matrices`: copied from its block's
    /// +0x740 by `draw_car`).
    pub part_matrices: [crate::memory::session::PlainFloat; 64],
    /// The drawn car's turn (`FrameWork.placed_matrices`' first: its block's +0x880).
    pub car_turn: [crate::memory::session::PlainFloat; 16],
    /// The material way of the object being drawn (`other_way_kind`).
    pub other_way_kind: PlainDword,
    /// The other kind and the distance of the object's fade (`fade_words`).
    pub fade_words: [PlainDword; 2],
    /// The object being drawn or set up (`object_number`).
    pub object_number: PlainDword,
    /// The drawn car's model's group masks (`COURSE.group_masks`: its parts' groups by slot, set by
    /// `draw_car` for each car, [13] .. [17] read by `list_object_face`; outside a race [13], [14]
    /// the known kinds' hidden groups).
    pub group_masks: [PlainDword; 18],
    /// The groups of the car's parts still on it (`group_mask`).
    pub shown_groups: PlainDword,
    /// The face group mask of the car being drawn (`group_mask`; -1 after a car).
    pub group_mask: PlainDword,
    /// The car's parts present (`parts_present`), each view's own.
    pub parts_present: PlainDword,
    /// The left light's byte (`light_left`), each view's own.
    pub light_left: PlainDword,
    /// The right light's byte (`light_right`), each view's own.
    pub light_right: PlainDword,
}

impl CarDrawState {
    pub const fn new() -> Self {
        Self {
            car_states: [const { PlainDword::new(0) }; crate::race::blocks_store::BLOCK_COUNT],
            cars_drawn: PlainDword::new(0),
            car_kept: [const { PlainDword::new(0) }; 2],
            car_depth: PlainDword::new(0),
            car_draw_kept: PlainDword::new(0),
            car_draw_word: PlainDword::new(0),
            car_nearness: crate::memory::session::PlainFloat::new(0.0),
            groups_which: PlainDword::new(0),
            part_matrices: [const { crate::memory::session::PlainFloat::new(0.0) }; 64],
            car_turn: [const { crate::memory::session::PlainFloat::new(0.0) }; 16],
            other_way_kind: PlainDword::new(0),
            fade_words: [const { PlainDword::new(0) }; 2],
            object_number: PlainDword::new(0),
            group_masks: [const { PlainDword::new(0) }; 18],
            shown_groups: PlainDword::new(0),
            group_mask: PlainDword::new(0),
            parts_present: PlainDword::new(0),
            light_left: PlainDword::new(0),
            light_right: PlainDword::new(0),
        }
    }
}

impl CarDrawState {
    /// The state bits of the car in block `block`.
    pub fn state_of(&self, block: usize) -> Result<&PlainDword, crate::errors::BadInput> {
        self.car_states.get(block).ok_or(crate::errors::BadInput(
            "a car's state bits not of a car block",
        ))
    }
}
