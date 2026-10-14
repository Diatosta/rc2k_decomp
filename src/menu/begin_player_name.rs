//! `menu::begin_player_name` (0x467C34): typing a player's name begins.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Unless the Enter came with ah = 4 (`dword_7554D4` at 1): the current player's name
//! kept (`menu/player_name.rs`), cleared (`menu::clear_text`), and the entry going on
//! (`dword_7556A8` = 1); then, either way, the length typed (`dword_75556D`) 0. The
//! action of the name's item of the four players' details pages.
//! Refuses no current player or a player without a name.
//! Compared in shadow mode.
//! Self-test: the four players, both kinds of Enter.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::clear_text::clear_text;
use crate::menu::player::{PLAYERS, Player, player_at};
use crate::menu::player_name::{keep_name, kept_region, name};
use crate::menu::text_entry::{ENTERING_RVA, LENGTH_RVA, TEXT_LEN, set_length};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::state::PLAYER_INDEX_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::begin_player_name", rva(0x467C34), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ENTER_WITH_4_RVA: usize = rva(0x7554D4); // dword_7554D4

// ---- Rust version ----

pub unsafe fn begin_player_name() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(ENTER_WITH_4_RVA) != 1 {
            let name = name()?;
            keep_name(name);
            clear_text(name);
            game_write(ENTERING_RVA, 1u32);
        }
    }
    set_length(0);
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA), at(ENTERING_RVA), kept_region()];
    if let Ok(name) = name() {
        regions.push((name as u32, TEXT_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_player_name() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(PLAYER_INDEX_RVA), at(ENTER_WITH_4_RVA)];
    let saved_inputs = capture(&inputs);
    for index in 0..PLAYERS {
        let player = player_at(index);
        if player == 0 || unsafe { (player as *const Player).read_unaligned() }.name == 0 {
            continue;
        }
        unsafe { game_write(PLAYER_INDEX_RVA, index) };
        let regions = regions();
        let saved = capture(&regions);
        for enter in [0u32, 1, 4] {
            check_state(
                &mut t,
                &format!("player {index}, dword_7554D4 {enter}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved);
                    game_write(ENTER_WITH_4_RVA, enter);
                    game_write(LENGTH_RVA, 7u32);
                    game_write(ENTERING_RVA, 0u32);
                },
                &regions,
                &rust,
            );
        }
        restore(&regions, &saved);
    }
    restore(&inputs, &saved_inputs);
    t.finish()
}
