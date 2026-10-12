//! `saves::read_replay_slot` (0x41ECAD): reads a replay's head,
//! `savedata\replay\replayNN`, and copies its name into the replay list.
//!
//! In:  ebx = slot, 0 .. 99
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! Up to 0x430 bytes to `dword_528504`. Accepted if the read did not fail, the first dword
//! is "REPL" and the dword at +8 is 0x50400; the 0x20 bytes at +0x10 then go to
//! `byte_528934[slot * 0x20]`. A short file is not noticed as such: what the buffer held
//! before decides (as in `saves::read_savegame_slot`).
//! One caller, `sub_411239`, for all 100 slots at start-up. Refuses a slot past 99.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::{buffer as path_buffer, numbered_path_in_game};
use crate::files::read_file::{LAST_FILE_PATH_RVA, read_file};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, hook_asm, rva};
use crate::menu::name_lists::{NAME_LEN, REPLAY_NAMES_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::read_replay_slot", rva(0x41ECAD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TEMPLATE_RVA: usize = rva(0x52974B); // "savedata\replay\replay00"
const HEAD_RVA: usize = rva(0x528504); // dword_528504
const HEAD_LEN: usize = 0x430;
const MAGIC: u32 = 0x4C50_4552; // "REPL"
const VERSION: u32 = 0x50400;
const SLOTS: u32 = 100;

fn slot_name(slot: u32) -> *mut u8 {
    game_addr(REPLAY_NAMES_RVA + slot as usize * NAME_LEN) as *mut u8
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub unsafe fn read_replay_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    unsafe {
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        let head: *mut u8 = game_ptr(HEAD_RVA);
        let read = read_file(path, head, HEAD_LEN as u32, 0)?;
        if !read.ok
            || game_read::<u32>(HEAD_RVA) != MAGIC
            || game_read::<u32>(HEAD_RVA + 8) != VERSION
        {
            return Ok(false);
        }
        std::ptr::copy(head.add(0x10), slot_name(slot), NAME_LEN);
        Ok(true)
    }
}

// ---- hook ----

fn regions(slot: u32) -> Vec<Region> {
    let mut regions = vec![
        (game_addr(HEAD_RVA) as u32, HEAD_LEN),
        (path_buffer() as u32, 0x100),
        at(LAST_FILE_PATH_RVA),
    ];
    if slot < SLOTS {
        regions.push((slot_name(slot) as u32, NAME_LEN));
    }
    regions
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ok = unsafe { read_replay_slot(r.ebx) }?;
    Ok(Left::carry(!ok))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebx), &rust);
}

// ---- self-test: every slot, with whatever replays the owner has ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for slot in 0..SLOTS {
        let regions = regions(slot);
        let saved = capture(&regions);
        check_state(
            &mut t,
            &format!("slot={slot}"),
            call_original,
            Regs {
                ebx: slot,
                ..Regs::SENTINEL
            },
            &|| restore(&regions, &saved),
            &regions,
            &rust,
        );
        restore(&regions, &saved);
    }
    t.finish()
}
