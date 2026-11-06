//! `screens::service_box_frame_4` (0x4932D4): the yellow frame round the chosen part of the
//! service area's second row of six cells (`service_boxes::chosen_frame`, item 0xF).
//!
//! Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::state_store::MENU_STATE;
use crate::screens::service_boxes::chosen_frame;

pub fn service_box_frame_4() -> Result<(), Refused> {
    chosen_frame(
        0xF,
        &MENU_STATE.driver_stats.setups[1],
        &MENU_LISTS.inspection_flags_a[0xF],
        0x193,
        0x37,
        6,
    )
}
