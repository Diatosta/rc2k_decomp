//! The yellow frame round the chosen part of the service area's second row of six cells
//! (item 0xF). Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::chosen_frame;

pub fn service_box_frame_4(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    chosen_frame(
        o,
        0xF,
        &menus.screens.state.driver_stats.setups[1],
        &menus.tables.lists.inspection_flags_a[0xF],
        0x193,
        0x37,
        6,
    )
}
