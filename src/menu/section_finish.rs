//! `menu::section_finish` (0x4A9439): the championship finish routine for the first player
//! done with the section (`menu::find_section_player`; none: nothing but a key taken):
//! `dword_75B108` and `dword_75ACE4` 0, the race's set-up from it
//! (`finish_calls::set_up`), then the sounds' and screens' set-up, `drive::finish_race_0`,
//! and the front end's set-up again (`finish_calls::CALLED` 1 .. 13); a key taken at the
//! end (eax 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60` (twice). The finding is its Rust version; the calls through
//! `part`, in the game through their addresses (hooks where ported). Ungated (the finish
//! routine runs). Refuses what the finding refuses, before anything. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins, against the Rust
//! version, the done player by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::menu::finish_calls::{Game, PLACES, Part, game_part, regions};
use crate::menu::standings::*;
use crate::service::harness::{at, capture, restore};
use rc2k_core::menu::finish;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::section_finish", rva(0x4A9439), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn section_finish(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::finish::section_finish`.
    finish::section_finish(&mut Game { g: *entry, part })?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { section_finish(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { section_finish(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4A9439,
        0xB0,
        &[
            0x60, 0xC7, 0x05, 0x08, 0xB1, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x15, 0x57, 0x5C, 0x61, 0x66, 0x6B, 0x70, 0x75, 0x7A, 0x7F, 0x84, 0x89, 0x8E, 0x93,
            0x98, 0x9D, 0xA2, 0xA9,
        ],
        &PLACES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    if (0..4).any(|n| player(n) == 0) {
        return t.skip("the game's players are not there");
    }
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([SECTION_RVA, PLAYER_COUNT_RVA, rva(0x7556D0)].map(at));
    vars.extend((0..4u32).map(|n| (player(n), 0x100)));

    let saved = capture(&vars);
    for case in 0..5u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SECTION_RVA, case % 3);
                for n in 0..4u32 {
                    put(
                        player(n) + 0xB4 + 4 * (case % 3),
                        if n == case % 4 { 0x14 } else { 0 },
                    );
                }
            },
            &|r| unsafe { section_finish(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
