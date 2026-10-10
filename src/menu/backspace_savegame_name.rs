//! `menu::backspace_savegame_name` (0x46D5FD): takes the last character off the name of the
//! highlighted saved game: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! In:  nothing
//! Out: every register preserved
//! Also called by the replay name editor, which is a slip in the original (see
//! `edit_replay_name`). Refuses a name past the list and a length outside 0 .. 0x1F.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::name_lists::{NAME_LEN, SAVEGAME_PAGE_RVA, savegame_name};
use crate::menu::text_entry::{LENGTH_RVA, length, set_length};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::backspace_savegame_name",
    rva(0x46D5FD),
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

// ---- Rust version ----

pub unsafe fn backspace_savegame_name() -> Result<(), BadInput> {
    let len = length();
    if len == 0 {
        return Ok(());
    }
    let name = savegame_name()?;
    if !(0..NAME_LEN as i32).contains(&len) {
        return Err(BadInput("text length outside the name"));
    }
    set_length(len - 1);
    unsafe { name.add(len as usize - 1).write(0) };
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA)];
    if let Ok(name) = savegame_name() {
        regions.push((name as u32, NAME_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { backspace_savegame_name() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the last name of the list (page 9, item 10), which is put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(SAVEGAME_PAGE_RVA), at(MENU_ITEM_RVA)];
    let saved_inputs = capture(&inputs);
    unsafe {
        game_write(SAVEGAME_PAGE_RVA, 9u32);
        game_write(MENU_ITEM_RVA, 10u32);
    }
    let regions = regions();
    let saved = capture(&regions);
    if let Ok(name) = savegame_name() {
        for len in [0i32, 1, 5, 30, 31] {
            check_state(
                &mut t,
                &format!("length={len}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    name.write_bytes(b'n', NAME_LEN);
                    set_length(len);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
