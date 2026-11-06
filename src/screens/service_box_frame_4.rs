//! `screens::service_box_frame_4` (0x4932D4): the yellow frame round the chosen part of the
//! service area's second row of six cells (`service_boxes::chosen_frame`, item 0xF).
//!
//! Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::screens::service_boxes::chosen_frame;

pub fn service_box_frame_4() -> Result<(), Refused> {
    let menus = menus();
    chosen_frame(
        0xF,
        &menus.screens.state.driver_stats.setups[1],
        &menus.tables.lists.inspection_flags_a[0xF],
        0x193,
        0x37,
        6,
    )
}
