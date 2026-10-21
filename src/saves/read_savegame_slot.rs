//! `saves::read_savegame_slot` (0x4AB4FE): reads a saved game, `savedata\savegame\svgameNN`,
//! into the buffer at 0x51EBB4 and copies its name into the saved game list.
//!
//! In:  ebx = slot, 0 .. 99 (eax is saved and compared but has no effect)
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! It reads up to 0x8000 bytes. The file is accepted if the read did not fail, the first
//! dword is "R5SG" and the dword at +8 is 0x51122; the 0x20 bytes at +0x10 then go to
//! `byte_526BB4[slot * 0x20]`. A short file is not noticed as such: what the buffer held
//! before decides (reproduced). It only fills the buffer: putting the state back into the
//! game is done by its callers `sub_46D88F` and `sub_46E6E1`, which are not ported.
//! Called for all 100 slots at start-up (`sub_411239`). Refuses a slot past 99.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::{buffer as path_buffer, numbered_path_in_game};
use crate::files::read_file::{LAST_FILE_PATH_RVA, read_file};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_ptr, hook_asm, rva};
use crate::menu::name_lists::NAME_LEN;
use crate::saves::write_savegame_slot::{FILE_LEN, FILE_RVA, SLOTS, TEMPLATE_RVA, slot_name};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::read_savegame_slot", rva(0x4AB4FE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub unsafe fn read_savegame_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    unsafe {
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        let file: *mut u8 = game_ptr(FILE_RVA);
        let read = read_file(path, file, FILE_LEN as u32, 0)?;
        // Converted: `rc2k_core::saves::kept::slot_name`.
        use rc2k_core::saves::kept;
        let block = std::slice::from_raw_parts(game_ptr::<u8>(FILE_RVA), 0x30);
        match kept::slot_name(block, kept::SAVEGAME, kept::SAVEGAME_VERSION) {
            Some(name) if read.ok => {
                std::ptr::copy(name.as_ptr(), slot_name(slot), NAME_LEN);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

// ---- hook ----

/// What reading slot `slot` may write.
fn regions(slot: u32) -> Vec<Region> {
    let mut regions = vec![
        (game_addr(FILE_RVA) as u32, FILE_LEN),
        (path_buffer() as u32, 0x100),
        at(LAST_FILE_PATH_RVA),
    ];
    if slot < SLOTS {
        regions.push((slot_name(slot) as u32, NAME_LEN));
    }
    regions
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ok = unsafe { read_savegame_slot(r.ebx) }?;
    Ok(Left::carry(!ok))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebx), &rust);
}

// ---- self-test: every slot, with whatever saved games the owner has ----
//
// Slots without a file take the failing path. Files with a wrong head are only covered if
// such files happen to exist.

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for slot in 0..SLOTS {
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
