//! `screens::service_box_frame_2` (0x491987): the yellow frame round the chosen part of the
//! service area's first row of five cells (`service_boxes::chosen_frame`, item 0xE).
//!
//! Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::state_store::MENU_STATE;
use crate::screens::service_boxes::chosen_frame;

pub fn service_box_frame_2() -> Result<(), Refused> {
    chosen_frame(
        0xE,
        &MENU_STATE.driver_stats.setups[0],
        &MENU_LISTS.inspection_flags_a[0xE],
        0x44,
        0x42,
        5,
    )
}
