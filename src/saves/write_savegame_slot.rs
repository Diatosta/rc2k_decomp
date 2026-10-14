//! `saves::write_savegame_slot` (0x4AB558): builds a saved game in the buffer at 0x51EBB4 and
//! writes it to `savedata\savegame\svgameNN`.
//!
//! In:  ebx = slot, 0 .. 99
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! The file is always 0x8000 bytes: "R5SG", 0, version 0x51122, `packed_local_time`, the
//! slot's 0x20-byte name from the saved game list (`byte_526BB4`), then what
//! `gather_savegame` collects (0x725C bytes); the rest of the buffer is written as it is.
//! Only caller: `menu::edit_savegame_name`.
//! Refuses a slot past 99 and what `gather_savegame` refuses.
//! Shadow mode writes the file twice; the time stamp is left out of the comparison.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::{buffer as path_buffer, numbered_path_in_game};
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::files::write_file::write_file;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_ptr, game_write, hook_asm, rva};
use crate::menu::name_lists::{NAME_LEN, SAVEGAME_NAMES_RVA};
use crate::saves::gather_savegame::gather_savegame;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state_with, dispatch_state, restore};
use crate::system::packed_local_time::packed_local_time;
use std::ptr::copy_nonoverlapping;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::write_savegame_slot",
    rva(0x4AB558),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const FILE_RVA: usize = rva(0x51EBB4); // dword_51EBB4: the saved game as it is written
pub const FILE_LEN: usize = 0x8000;
pub const TEMPLATE_RVA: usize = rva(0x5296FB); // aSavedataSavega: "savedata\savegame\svgame00"
pub const SLOTS: u32 = 100;

pub const MAGIC: u32 = 0x4753_3552; // "R5SG"
pub const VERSION: u32 = 0x5_1122;

pub fn slot_name(slot: u32) -> *mut u8 {
    game_ptr::<u8>(SAVEGAME_NAMES_RVA).wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the file was written.
pub unsafe fn write_savegame_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    unsafe {
        let file: *mut u8 = game_ptr(FILE_RVA);
        // First, because it is what can be refused: nothing else is written before it.
        gather_savegame(file.add(0x30))?;
        game_write(FILE_RVA, MAGIC);
        game_write(FILE_RVA + 4, 0u32);
        game_write(FILE_RVA + 8, VERSION);
        game_write(FILE_RVA + 0xC, packed_local_time());
        copy_nonoverlapping(slot_name(slot), file.add(0x10), NAME_LEN);
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        Ok(write_file(path, file, FILE_LEN as u32)?.ok)
    }
}

// ---- hook ----

/// What writing a slot leaves behind in memory, without the time stamp at +0xC. (What
/// `gather_savegame` changes in the cars is its own hook's business.)
fn regions() -> [Region; 4] {
    let file = game_addr(FILE_RVA) as u32;
    [
        (file, 0xC),
        (file + 0x10, FILE_LEN - 0x10),
        (path_buffer() as u32, 0x100),
        at(LAST_FILE_PATH_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ok = unsafe { write_savegame_slot(r.ebx) }?;
    Ok(Left::carry(!ok))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: slot 99, whose file and name are put back afterwards ----

const TEST_SLOT: u32 = 99;
const TEST_FILE: &str = "savedata\\savegame\\svgame99";

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
    if unsafe {
        gather_savegame(vec![0u8; crate::saves::gather_savegame::GATHERED_LEN].as_mut_ptr())
    }
    .is_err()
    {
        return t.skip("the players are not there");
    }
    let old_file = std::fs::read(TEST_FILE).ok();
    let inputs = [(slot_name(TEST_SLOT) as u32, NAME_LEN)];
    let saved = (capture(&regions()), capture(&inputs));
    let before = Regs {
        ebx: TEST_SLOT,
        ..Regs::SENTINEL
    };

    // If the folder is missing both versions fail, which is a case too.
    for name in [&b"rc2k_hook selftest save\0"[..], &b"two\0"[..]] {
        let prepare = || unsafe {
            let entry = slot_name(TEST_SLOT);
            entry.write_bytes(0xAA, NAME_LEN);
            copy_nonoverlapping(name.as_ptr(), entry, name.len());
            game_ptr::<u8>(FILE_RVA).write_bytes(0xAA, FILE_LEN);
        };
        check_state_with(
            &mut t,
            &format!(
                "name={:?}",
                String::from_utf8_lossy(&name[..name.len() - 1])
            ),
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
