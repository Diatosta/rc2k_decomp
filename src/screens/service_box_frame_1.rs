//! The orange frame round the installed part of the service area's first row of five cells.
//! Part of `screens::draw_service_boxes`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::{ORANGE, cell_frame};
use crate::service::owner::service;

pub fn service_box_frame_1(o: Owners<'static>) -> Result<(), Refused> {
    let installed = service().setups[0].installed.get();
    if installed < 5 {
        cell_frame(o, 0x44, 0x42, installed, ORANGE)?;
    }
    Ok(())
}
