//! A view's own camera values and the car it draws: what a race view's drawing writes and then
//! reads in `ViewVars` (`race::view_vars_store`). One per view slot
//! (`race::view_scratch::ViewScratch::state`). The shared fields are still written beside them
//! for what reads them outside the drawing (the effects, the front end).

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells)]
pub struct ViewState {
    /// The car being drawn.
    pub drawn_car: PlainDword,
    /// The car's switch share.
    pub view_car_flag_a: PlainFloat,
    /// The car's camera view.
    pub view_car_stage: PlainDword,
    /// The view's place: x, y, z.
    pub view_place: [PlainDword; 3],
    /// The view's angles: two triples.
    pub view_angles: [PlainDword; 6],
}

impl ViewState {
    /// The shared stores' starting values.
    pub const fn new() -> Self {
        let crate::race::view_vars_store::ViewVars {
            drawn_car,
            view_car_flag_a,
            view_car_stage,
            view_place,
            view_angles,
            ..
        } = crate::race::view_vars_store::ViewVarsCopy::new();
        Self {
            drawn_car,
            view_car_flag_a,
            view_car_stage,
            view_place,
            view_angles,
        }
    }
}
