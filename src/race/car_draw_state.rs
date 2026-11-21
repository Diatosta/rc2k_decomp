//! A view's cars' drawing state: what drawing the cars and the objects works out and reads again
//! within the view, one in each view's slot (`view_scratch`) and one for the front end: the cars
//! drawn, the car's kept place and depth, the car's kept word and drawing word, the object's
//! material way, the fade words, the number of the object being drawn, the car's group masks and
//! face group mask, and each car block's state bits as the view draws them.

use crate::race::plain_slot::{
    Merge, PlainSlot, merge_float, merge_floats, merge_run, merge_value,
};

/// A view's cars' drawing values, plain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarDrawValues {
    /// Each car block's state bits as the view draws it (lights, seen
    /// from inside 0x10, casts a shadow, ...), set at the view's start (`car_light_modes`), by
    /// block number.
    pub car_states: [u32; crate::race::blocks_store::BLOCK_COUNT],
    /// The cars drawn this frame (`cars_drawn`).
    pub cars_drawn: u32,
    /// The car's kept place (`car_kept` ..).
    pub car_kept: [u32; 2],
    /// The car's depth (`car_depth`, 0x50000 at first).
    pub car_depth: u32,
    /// The car's kept word (0x2100000 while a car is drawn).
    pub car_draw_kept: u32,
    /// The car's drawing word (0x60 plus its nearness size).
    pub car_draw_word: u32,
    /// The drawn car's nearness, 0 .. 1 (`car_nearness`).
    pub car_nearness: f32,
    /// The drawn car's mode (`groups_which`: its block's `mode_on`), read by the face tests
    /// (`list_object_face`).
    pub groups_which: u32,
    /// The drawn car's four parts' matrices (`FrameWork.part_matrices`: copied from its block's
    /// `part_matrices` by `draw_car`).
    pub part_matrices: [f32; 64],
    /// The drawn car's turn (`FrameWork.placed_matrices`' first: its block's `place_matrix`).
    pub car_turn: [f32; 16],
    /// The material way of the object being drawn (`other_way_kind`).
    pub other_way_kind: u32,
    /// The other kind and the distance of the object's fade (`fade_words`).
    pub fade_words: [u32; 2],
    /// The object being drawn or set up (`object_number`).
    pub object_number: u32,
    /// The drawn car's model's group masks (`COURSE.group_masks`: its parts' groups by slot, set by
    /// `draw_car` for each car, [13] .. [17] read by `list_object_face`; outside a race [13], [14]
    /// the known kinds' hidden groups).
    pub group_masks: [u32; 18],
    /// The groups of the car's parts still on it (`group_mask`).
    pub shown_groups: u32,
    /// The face group mask of the car being drawn (`group_mask`; -1 after a car).
    pub group_mask: u32,
    /// The car's parts present (`parts_present`), each view's own.
    pub parts_present: u32,
    /// The left light's byte (`light_left`), each view's own.
    pub light_left: u32,
    /// The right light's byte (`light_right`), each view's own.
    pub light_right: u32,
}

impl CarDrawValues {
    pub const fn new() -> Self {
        Self {
            car_states: [0; crate::race::blocks_store::BLOCK_COUNT],
            cars_drawn: 0,
            car_kept: [0; 2],
            car_depth: 0,
            car_draw_kept: 0,
            car_draw_word: 0,
            car_nearness: 0.0,
            groups_which: 0,
            part_matrices: [0.0; 64],
            car_turn: [0.0; 16],
            other_way_kind: 0,
            fade_words: [0; 2],
            object_number: 0,
            group_masks: [0; 18],
            shown_groups: 0,
            group_mask: 0,
            parts_present: 0,
            light_left: 0,
            light_right: 0,
        }
    }

    /// The state bits of the car in block `block`.
    pub fn state_of(&self, block: usize) -> Result<u32, crate::errors::BadInput> {
        self.car_states
            .get(block)
            .copied()
            .ok_or(crate::errors::BadInput(
                "a car's state bits not of a car block",
            ))
    }

    /// Sets the state bits of the car in block `block`.
    pub fn set_state(&mut self, block: usize, bits: u32) -> Result<(), crate::errors::BadInput> {
        *self
            .car_states
            .get_mut(block)
            .ok_or(crate::errors::BadInput(
                "a car's state bits not of a car block",
            ))? = bits;
        Ok(())
    }
}

impl Merge for CarDrawValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.car_states, &now.car_states, &start.car_states);
        merge_value(&mut self.cars_drawn, &now.cars_drawn, &start.cars_drawn);
        merge_run(&mut self.car_kept, &now.car_kept, &start.car_kept);
        merge_value(&mut self.car_depth, &now.car_depth, &start.car_depth);
        merge_value(
            &mut self.car_draw_kept,
            &now.car_draw_kept,
            &start.car_draw_kept,
        );
        merge_value(
            &mut self.car_draw_word,
            &now.car_draw_word,
            &start.car_draw_word,
        );
        merge_float(
            &mut self.car_nearness,
            &now.car_nearness,
            &start.car_nearness,
        );
        merge_value(
            &mut self.groups_which,
            &now.groups_which,
            &start.groups_which,
        );
        merge_floats(
            &mut self.part_matrices,
            &now.part_matrices,
            &start.part_matrices,
        );
        merge_floats(&mut self.car_turn, &now.car_turn, &start.car_turn);
        merge_value(
            &mut self.other_way_kind,
            &now.other_way_kind,
            &start.other_way_kind,
        );
        merge_run(&mut self.fade_words, &now.fade_words, &start.fade_words);
        merge_value(
            &mut self.object_number,
            &now.object_number,
            &start.object_number,
        );
        merge_run(&mut self.group_masks, &now.group_masks, &start.group_masks);
        merge_value(
            &mut self.shown_groups,
            &now.shown_groups,
            &start.shown_groups,
        );
        merge_value(&mut self.group_mask, &now.group_mask, &start.group_mask);
        merge_value(
            &mut self.parts_present,
            &now.parts_present,
            &start.parts_present,
        );
        merge_value(&mut self.light_left, &now.light_left, &start.light_left);
        merge_value(&mut self.light_right, &now.light_right, &start.light_right);
    }
}

/// A view's cars' drawing state.
pub type CarDrawState = PlainSlot<CarDrawValues>;
