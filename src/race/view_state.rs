//! A view's own camera values and the car it draws (frame plan F7): what a race view's
//! drawing writes and then reads in `ViewVars` (`race::view_vars_store`). One per view slot
//! (`race::view_scratch::ViewScratch::state`). The shared fields are still written beside
//! them for what reads them outside the drawing (the effects, the front end).

use crate::race::groups_store::ViewAndRecords;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewState {
    /// `g_drawnCar` (0x60627C): the car being drawn, each view's own.
    pub drawn_car: Dword,
    /// `view_car_flag_a` (0x60626C): the car's switch share, each view's own.
    pub view_car_flag_a: Dword,
    /// `view_car_stage` (0x606270): the car's camera view, each view's own.
    pub view_car_stage: Dword,
    /// `g_viewPlace` (0x606298) x, y, z, each view's own.
    pub view_place: [Dword; 3],
    /// `g_viewAngles` (0x6062B8): two triples, each view's own.
    pub view_angles: [Dword; 6],
}

impl ViewState {
    /// With the stores' first values.
    pub const fn new() -> Self {
        let ViewAndRecords {
            view_vars:
                crate::race::view_vars_store::ViewVars {
                    drawn_car,
                    view_car_flag_a,
                    view_car_stage,
                    view_place,
                    view_angles,
                    ..
                },
            ..
        } = ViewAndRecords::new();
        Self {
            drawn_car,
            view_car_flag_a,
            view_car_stage,
            view_place,
            view_angles,
        }
    }
}
