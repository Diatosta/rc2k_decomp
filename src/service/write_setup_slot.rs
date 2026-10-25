//! `service::write_setup_slot` (0x4AB378): builds the 0x48-byte car setup file in the buffer at
//! 0x527838 and writes it to `savedata\carsetup\carsetNN`.
//!
//! In:  ebx = slot, 0 .. 99
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! The file: "CRSU", 0, version 0x50000, `packed_local_time`, the slot's 0x20-byte name from
//! `byte_527880[slot * 0x20]`, then the chosen tyre, the car's brake balance, the chosen ride
//! height and suspension stiffness, the car's steering sensitivity and the chosen gear ratio.
//! The name is taken from the slot list, so the caller puts it there first; nothing takes it
//! out again if the write fails. Refuses a slot past 99 and a missing player.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_in_game;
use crate::files::write_file::write_file;
use crate::framework::game_ptr;
use crate::service::pages::{
    BRAKE_BALANCE, GEAR_RATIO, RIDE_HEIGHT, STEERING_SENSITIVITY, SUSPENSION_STIFFNESS, TYRE,
};
use crate::service::read_setup_slot::{
    NAME_LEN, SETUP_FILE_LEN, SETUP_FILE_RVA, SLOT_COUNT, TEMPLATE_RVA, slot_name,
};
use crate::service::state::current_player;
use crate::system::packed_local_time::packed_local_time;
use std::ptr::copy_nonoverlapping;

// ---- Rust version ----

/// True if the file was written.
pub unsafe fn write_setup_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    let Some(player) = current_player() else {
        return Err(BadInput("no current player object: the original reads through null").into());
    };
    unsafe {
        let file: *mut u8 = game_ptr(SETUP_FILE_RVA);
        // Converted: `rc2k_core::saves::kept::setup_file`.
        let in_car = |offset: u32| (player.wrapping_add(offset) as *const u32).read_unaligned();
        let bytes = rc2k_core::saves::kept::setup_file(
            packed_local_time(),
            &(slot_name(slot) as *const [u8; NAME_LEN]).read_unaligned(),
            [
                TYRE.change.chosen.get(),
                in_car(BRAKE_BALANCE.car_offset),
                RIDE_HEIGHT.change.chosen.get(),
                SUSPENSION_STIFFNESS.change.chosen.get(),
                in_car(STEERING_SENSITIVITY.car_offset),
                GEAR_RATIO.change.chosen.get(),
            ],
        );
        copy_nonoverlapping(bytes.as_ptr(), file, bytes.len());
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        Ok(write_file(path, file, SETUP_FILE_LEN as u32)?.ok)
    }
}
