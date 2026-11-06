//! `screens::service_box_frame_3` (0x49252A): the orange frame round the installed part of the
//! service area's second row of six cells.
//!
//! Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::menu::state_store::MENU_STATE;
use crate::screens::service_boxes::{ORANGE, cell_frame};

pub fn service_box_frame_3() -> Result<(), Refused> {
    let installed = MENU_STATE.driver_stats.setups[1].installed.get();
    if installed < 6 {
        cell_frame(0x193, 0x37, installed, ORANGE)?;
    }
    Ok(())
}
