//! `scene::assign_menu_textures` (0x490DF3): the eight slots of kind 0x258 get the
//! menu's own textures.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! For each of the eight numbers of `dword_75A622` that is not -1: the slot (kind
//! 0x258's first slot, `dword_5D2448`, plus its place in the eight) wants that
//! number plus kind 0x258's first texture (`dword_5D1448`).
//! Two callers.
//! Refuses a slot past the table.
//! Self-test: 12 settings by lot in the game's places, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::scene::textures::{
    KIND_SLOT_RVA, KIND_TEXTURE_RVA, SLOTS, WANTED_RVA, table, test_table, write_table,
};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::assign_menu_textures",
    rva(0x490DF3),
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

const OWN_RVA: usize = rva(0x75A622); // dword_75A622, eight numbers
const KIND: usize = 0x258;
const FIRST_TEXTURE_RVA: usize = KIND_TEXTURE_RVA + KIND * 4; // dword_5D1448
const FIRST_SLOT_RVA: usize = KIND_SLOT_RVA + KIND * 4; // dword_5D2448

// ---- Rust version ----

pub unsafe fn assign_menu_textures() -> Result<(), BadInput> {
    // Converted: `rc2k_core::scene::textures::menu_textures`.
    unsafe {
        let wanted = rc2k_core::scene::textures::menu_textures(
            game_read(OWN_RVA),
            game_read(FIRST_SLOT_RVA),
            game_read(FIRST_TEXTURE_RVA),
        )?;
        for (slot, texture) in wanted {
            game_write(WANTED_RVA + slot as usize * 4, texture);
        }
        Ok(())
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { assign_menu_textures() }?;
    Ok(Left::nothing())
}

/// The eight slots, where they lie in the table.
fn regions() -> Vec<Region> {
    let slot: u32 = unsafe { game_read(FIRST_SLOT_RVA) };
    if slot.checked_add(8).is_some_and(|end| end <= SLOTS) {
        vec![(game_addr(WANTED_RVA) as u32 + slot * 4, 32)]
    } else {
        Vec::new()
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [table(WANTED_RVA, SLOTS)];
    let inputs = [
        (game_addr(OWN_RVA) as u32, 32),
        at(FIRST_TEXTURE_RVA),
        at(FIRST_SLOT_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0003u32;
    for case in 0..12u32 {
        let own = test_table(&mut seed, 8, 1 + case % 3, 0x40);
        let slot = [0u32, 0x10, SLOTS - 8, 0x123][case as usize % 4];
        let texture = [0u32, 0x200, 0xFFFF_FFF0][case as usize % 3];
        check_state(
            &mut t,
            &format!("case {case}: {own:x?}, first slot {slot:#x}, first texture {texture:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                write_table(OWN_RVA, &own);
                game_write(FIRST_SLOT_RVA, slot);
                game_write(FIRST_TEXTURE_RVA, texture);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
