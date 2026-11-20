//! Builds the 0x48-byte car setup file in the set-ups' buffer and writes it to
//! `savedata\carsetup\carsetNN`.
//!
//! `slot` is 0 .. 99. The file: "CRSU", 0, version 0x50000, `packed_local_time`, the slot's
//! 0x20-byte name from the set-ups' names, then the chosen tyre, the car's brake balance, the
//! chosen ride height and suspension stiffness, the car's steering sensitivity and the chosen
//! gear ratio. The name is taken from the slot list, so the caller puts it there first and takes it
//! out again if the write fails (`edit_setup_name`). Refuses a slot past 99 and a missing player.

use crate::clock::packed_local_time::packed_local_time;
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

/// The set-up file's bytes: the chosen tyre, ride height, suspension stiffness and gear ratio
/// from the service's items, the brake balance and steering sensitivity from the car's player.
fn setup_file_bytes(
    service: &Service,
    time: u32,
    name: &[u8; NAME_LEN],
    player: rc2k_core::menu::player::Player,
) -> [u8; SETUP_FILE_LEN] {
    let in_car = |field: rc2k_core::menu::player::SetupField| field.get(player);
    rc2k_core::saves::kept::setup_file(
        time,
        name,
        [
            TYRE.change.chosen(service).get(),
            in_car(BRAKE_BALANCE.in_car),
            RIDE_HEIGHT.change.chosen(service).get(),
            SUSPENSION_STIFFNESS.change.chosen(service).get(),
            in_car(STEERING_SENSITIVITY.in_car),
            GEAR_RATIO.change.chosen(service).get(),
        ],
    )
}

/// True if the file was written.
pub(super) fn write_setup_slot(
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
        let bytes = setup_file_bytes(
            service,
            packed_local_time(),
            &slot_name(service, slot).read_array::<NAME_LEN>("a setup's name")?,
            player.get(),
        );
        file.write_at(0, &bytes, "the setup file")?;
        let template = service.template.read();
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        Ok(write_file_from(&path[..], &file.read()[..SETUP_FILE_LEN])?.ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::menu::player::Player;
    use rc2k_core::service::setups::SetupValues;

    /// The file the service's four items and the car's two values make reads back as the same
    /// six values, and writing those again gives the same bytes.
    #[test]
    fn the_setup_file_round_trips_byte_for_byte() {
        let _lock = crate::test_support::test_lock();
        let items = &crate::test_support::own::menus().service.setups;
        let kept: Vec<u32> = items.iter().map(|item| item.chosen.get()).collect();
        for (n, item) in items.iter().enumerate() {
            item.chosen.set(0x11 * (n as u32 + 1));
        }
        let mut player = Player::from_le_bytes([0; 0x28c]);
        player.brake_balance = 0x31;
        player.steering_sensitivity = 0x47;
        let name = [b's'; NAME_LEN];
        let bytes = setup_file_bytes(
            &crate::test_support::own::menus().service,
            0x1234_5678,
            &name,
            player,
        );
        for (item, value) in items.iter().zip(kept) {
            item.chosen.set(value);
        }
        let read = SetupValues::from_file(&bytes);
        // Item order: gear ratio, tyre, ride height, suspension stiffness.
        assert_eq!(
            [
                read.gear_ratio,
                read.tyre,
                read.ride_height,
                read.suspension_stiffness
            ],
            [0x11, 0x22, 0x33, 0x44]
        );
        assert_eq!(
            (read.brake_balance, read.steering_sensitivity),
            (0x31, 0x47)
        );
        assert_eq!(rc2k_core::saves::kept::setup_name(&bytes), Some(name));
        let again = rc2k_core::saves::kept::setup_file(
            0x1234_5678,
            &name,
            [
                read.tyre,
                read.brake_balance,
                read.ride_height,
                read.suspension_stiffness,
                read.steering_sensitivity,
                read.gear_ratio,
            ],
        );
        assert_eq!(bytes, again);
    }
}
