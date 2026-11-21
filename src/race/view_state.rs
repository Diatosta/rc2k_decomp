//! A view's own camera values and the car it draws: what a race view's drawing writes and then
//! reads in `ViewVars` (`race::view_vars_store`). One per view slot
//! (`race::view_scratch::ViewScratch::state`). The shared fields are still written beside them
//! for what reads them outside the drawing (the effects, the front end).

use crate::race::plain_slot::{Merge, merge_run, merge_value};

/// The values, plain; every one starts at 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewState {
    /// The car being drawn.
    pub drawn_car: u32,
    /// The car's switch share, as the float's bits.
    pub view_car_flag_a_bits: u32,
    /// The car's camera view.
    pub view_car_stage: u32,
    /// The view's place: x, y, z.
    pub view_place: [u32; 3],
    /// The view's angles: two triples.
    pub view_angles: [u32; 6],
}

impl ViewState {
    /// The starting values.
    pub const fn new() -> Self {
        Self {
            drawn_car: 0,
            view_car_flag_a_bits: 0,
            view_car_stage: 0,
            view_place: [0; 3],
            view_angles: [0; 6],
        }
    }
}

impl Merge for ViewState {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(&mut self.drawn_car, &now.drawn_car, &start.drawn_car);
        merge_value(
            &mut self.view_car_flag_a_bits,
            &now.view_car_flag_a_bits,
            &start.view_car_flag_a_bits,
        );
        merge_value(
            &mut self.view_car_stage,
            &now.view_car_stage,
            &start.view_car_stage,
        );
        merge_run(&mut self.view_place, &now.view_place, &start.view_place);
        merge_run(&mut self.view_angles, &now.view_angles, &start.view_angles);
    }
}
