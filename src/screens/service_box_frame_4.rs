//! The yellow frame round the chosen part of the service area's second row of six cells
//! (item 0xF). Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::chosen_frame;

pub(super) fn service_box_frame_4(o: Owners<'static>) -> Result<(), Refused> {
    chosen_frame(
        o,
        0xF,
        &o.menus.service.setups[1],
        &o.menus.service.repair_flags[0xF],
        0x193,
        0x37,
        6,
    )
}
