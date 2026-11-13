//! Builds the 0x48-byte car setup file in the set-ups' buffer and writes it to
//! `savedata\carsetup\carsetNN`.
//!
//! `slot` is 0 .. 99. The file: "CRSU", 0, version 0x50000, `packed_local_time`, the slot's
//! 0x20-byte name from the set-ups' names, then the chosen tyre, the car's brake balance, the
//! chosen ride height and suspension stiffness, the car's steering sensitivity and the chosen
//! gear ratio. The name is taken from the slot list, so the caller puts it there first; nothing
//! takes it out again if the write fails. Refuses a slot past 99 and a missing player.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::write_file::write_file_from;
use crate::menu::owner::Menus;
use crate::service::owner::Service;
use crate::service::pages::{
    BRAKE_BALANCE, GEAR_RATIO, RIDE_HEIGHT, STEERING_SENSITIVITY, SUSPENSION_STIFFNESS, TYRE,
};
use crate::service::read_setup_slot::{NAME_LEN, SETUP_FILE_LEN, SLOT_COUNT, slot_name};
use crate::service::state::current_player;
use crate::system::packed_local_time::packed_local_time;

/// True if the file was written.
pub fn write_setup_slot(
    menus: &Menus,
    service: &'static Service,
    slot: u32,
) -> Result<bool, Refused> {
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    let Some(player) = current_player(menus) else {
        return Err(BadInput("no current player object: the original reads through null").into());
    };
    {
        let file = &service.set_ups.file;
        let in_car = |field: rc2k_core::menu::player::SetupField| field.get(player.get());
        let bytes = rc2k_core::saves::kept::setup_file(
            packed_local_time(),
            &slot_name(service, slot).read_array::<NAME_LEN>("a setup's name")?,
            [
                TYRE.change.chosen(menus).get(),
                in_car(BRAKE_BALANCE.in_car),
                RIDE_HEIGHT.change.chosen(menus).get(),
                SUSPENSION_STIFFNESS.change.chosen(menus).get(),
                in_car(STEERING_SENSITIVITY.in_car),
                GEAR_RATIO.change.chosen(menus).get(),
            ],
        );
        file.write_at(0, &bytes, "the setup file")?;
        let template = service.template.read();
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        Ok(write_file_from(&path[..], &file.read()[..SETUP_FILE_LEN])?.ok)
    }
}
