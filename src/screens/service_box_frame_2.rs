//! The yellow frame round the chosen part of the service area's first row of five cells
//! (item 0xE). Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::chosen_frame;

pub fn service_box_frame_2(o: Owners<'static>) -> Result<(), Refused> {
    chosen_frame(
        o,
        0xE,
        &o.menus.service.setups[0],
        &o.menus.service.repair_flags[0xE],
        0x44,
        0x42,
        5,
    )
}
