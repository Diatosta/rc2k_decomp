//! `service::read_setup_slot` (0x4AB31E): reads the head of a saved car setup,
//! `savedata\carsetup\carsetNN`, into the buffer at 0x527838 and copies its name into the
//! slot name table.
//!
//! In:  ebx = slot, 0 .. 99 (eax is saved and compared but has no effect)
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! It reads 0x48 bytes. The file is accepted if the read did not fail, the first dword is
//! "CRSU" and the dword at +8 is 0x50000; the 0x20 bytes at +0x10 then go to
//! `byte_527880[slot * 0x20]`. A file shorter than 0x48 bytes is not noticed as such: what the
//! buffer held before decides (reproduced). Refuses a slot past the table's 100 entries.
//! Called for all 100 slots at start-up (`sub_411239`) and by `load_setup_slot`.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::{buffer as path_buffer, numbered_path_in_game};
use crate::files::read_file::{LAST_FILE_PATH_RVA, read_file};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_ptr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::ptr::copy_nonoverlapping;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::read_setup_slot", rva(0x4AB31E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const TEMPLATE_RVA: usize = rva(0x529716); // aSavedataCarset: "savedata\carsetup\carset00"
pub const SETUP_FILE_RVA: usize = rva(0x527838); // dword_527838: the head of a setup file
pub const SETUP_FILE_LEN: usize = 0x48;
const SLOT_NAMES_RVA: usize = rva(0x527880); // byte_527880[100][0x20]
pub const SLOT_COUNT: u32 = 100;
pub const NAME_LEN: usize = 0x20;

pub const MAGIC: u32 = 0x5553_5243; // "CRSU"
pub const VERSION: u32 = 0x5_0000;

pub fn slot_name(slot: u32) -> *mut u8 {
    game_ptr::<u8>(SLOT_NAMES_RVA).wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub unsafe fn read_setup_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    unsafe {
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        let file: *mut u8 = game_ptr(SETUP_FILE_RVA);
        let read = read_file(path, file, SETUP_FILE_LEN as u32, 0)?;
        // Converted: `rc2k_core::saves::kept::setup_name`.
        let name = rc2k_core::saves::kept::setup_name(
            &(file as *const [u8; SETUP_FILE_LEN]).read_unaligned(),
        );
        match name {
            Some(name) if read.ok => {
                copy_nonoverlapping(name.as_ptr(), slot_name(slot), NAME_LEN);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

// ---- hook ----

/// What reading slot `slot` may write.
pub fn regions(slot: u32) -> Vec<Region> {
    let mut regions = vec![
        (game_addr(SETUP_FILE_RVA) as u32, SETUP_FILE_LEN),
        (path_buffer() as u32, 0x100),
        at(LAST_FILE_PATH_RVA),
    ];
    if slot < SLOT_COUNT {
        regions.push((slot_name(slot) as u32, NAME_LEN));
    }
    regions
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ok = unsafe { read_setup_slot(r.ebx) }?;
    Ok(Left::carry(!ok))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebx), &rust);
}

// ---- self-test: every slot, with whatever setup files the owner has saved ----
//
// Slots without a file take the failing path. Files with a wrong head or too short are only
// covered if such files happen to exist.

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for slot in 0..SLOT_COUNT {
        let regions = regions(slot);
        let saved = capture(&regions);
        let before = Regs {
            ebx: slot,
            ..Regs::SENTINEL
        };
        check_state(
            &mut t,
            &format!("slot={slot}"),
            call_original,
            before,
            &|| restore(&regions, &saved),
            &regions,
            &rust,
        );
        restore(&regions, &saved);
    }
    t.finish()
}
