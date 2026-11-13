//! The yellow frame round the chosen part of the service area's first row of five cells
//! (item 0xE). Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::chosen_frame;

pub fn service_box_frame_2(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    chosen_frame(
        o,
        0xE,
        &menus.screens.state.driver_stats.setups[0],
        &menus.tables.lists.inspection_flags_a[0xE],
        0x44,
        0x42,
        5,
    )
}
