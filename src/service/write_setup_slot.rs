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
//! Shadow mode writes the file twice; the time stamp is left out of the comparison.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::{buffer as path_buffer, numbered_path_in_game};
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::files::write_file::write_file;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state_with, dispatch_state, restore};
use crate::service::pages::{
    BRAKE_BALANCE, GEAR_RATIO, RIDE_HEIGHT, STEERING_SENSITIVITY, SUSPENSION_STIFFNESS, TYRE,
};
use crate::service::read_setup_slot::{
    NAME_LEN, SETUP_FILE_LEN, SETUP_FILE_RVA, SLOT_COUNT, TEMPLATE_RVA, slot_name,
};
use crate::service::state::current_player;
use crate::system::packed_local_time::packed_local_time;
use std::ptr::copy_nonoverlapping;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::write_setup_slot", rva(0x4AB378), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
                game_read(TYRE.change.chosen_rva),
                in_car(BRAKE_BALANCE.car_offset),
                game_read(RIDE_HEIGHT.change.chosen_rva),
                game_read(SUSPENSION_STIFFNESS.change.chosen_rva),
                in_car(STEERING_SENSITIVITY.car_offset),
                game_read(GEAR_RATIO.change.chosen_rva),
            ],
        );
        copy_nonoverlapping(bytes.as_ptr(), file, bytes.len());
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        Ok(write_file(path, file, SETUP_FILE_LEN as u32)?.ok)
    }
}

// ---- hook ----

/// What writing a slot leaves behind in memory, without the time stamp at +0xC.
fn regions() -> [Region; 4] {
    let file = game_addr(SETUP_FILE_RVA) as u32;
    [
        (file, 0xC),
        (file + 0x10, SETUP_FILE_LEN - 0x10),
        (path_buffer() as u32, 0x100),
        at(LAST_FILE_PATH_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ok = unsafe { write_setup_slot(r.ebx) }?;
    Ok(Left::carry(!ok))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: slot 99, whose file and name are put back afterwards ----

const TEST_SLOT: u32 = 99;
const TEST_FILE: &str = "savedata\\carsetup\\carset99";

/// The test slot's file without its time stamp, or one byte if there is none; the file is
/// removed, so that the next run has to write it again.
fn written_file() -> Vec<u8> {
    let Ok(mut bytes) = std::fs::read(TEST_FILE) else {
        return vec![0xFF];
    };
    let _ = std::fs::remove_file(TEST_FILE);
    if bytes.len() >= 0x10 {
        bytes[0xC..0x10].fill(0);
    }
    bytes
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if current_player().is_none() {
        return t.skip("there is no current player object");
    }
    let old_file = std::fs::read(TEST_FILE).ok();
    let inputs = [
        (slot_name(TEST_SLOT) as u32, NAME_LEN),
        at(TYRE.change.chosen_rva),
        at(RIDE_HEIGHT.change.chosen_rva),
        at(SUSPENSION_STIFFNESS.change.chosen_rva),
        at(GEAR_RATIO.change.chosen_rva),
    ];
    let saved = (capture(&regions()), capture(&inputs));
    let before = Regs {
        ebx: TEST_SLOT,
        ..Regs::SENTINEL
    };

    // If the folder is missing both versions fail, which is a case too.
    for (name, values) in [
        (&b"rc2k_hook selftest one\0"[..], [1u32, 20, 30, 2]),
        (&b"two\0"[..], [5, 100, 0, 4]),
    ] {
        let prepare = || unsafe {
            let entry = slot_name(TEST_SLOT);
            entry.write_bytes(0xAA, NAME_LEN);
            copy_nonoverlapping(name.as_ptr(), entry, name.len());
            game_write(TYRE.change.chosen_rva, values[0]);
            game_write(RIDE_HEIGHT.change.chosen_rva, values[1]);
            game_write(SUSPENSION_STIFFNESS.change.chosen_rva, values[2]);
            game_write(GEAR_RATIO.change.chosen_rva, values[3]);
            game_ptr::<u8>(SETUP_FILE_RVA).write_bytes(0xAA, SETUP_FILE_LEN);
        };
        check_state_with(
            &mut t,
            &format!("values={values:?}"),
            call_original,
            before,
            &prepare,
            &regions(),
            &rust,
            &written_file,
        );
    }

    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    match old_file {
        Some(bytes) => {
            let _ = std::fs::write(TEST_FILE, bytes);
        }
        None => {
            let _ = std::fs::remove_file(TEST_FILE);
        }
    }
    t.finish()
}
