//! `menu::championship_finish` (0x4A14CB): the championship finish routine. `dword_75B108`
//! and `dword_75ACE4` 0; the leader looked for in standings table B (event mode 0x13:
//! `menu::first_player_b` in group 0; otherwise `menu::first_player_unplaced`): when its
//! edx is 1, `dword_75ACCC` and `dword_75B108` 1 and the player `g_players`[ecx] (ecx as
//! the finder leaves it: section 16); otherwise `dword_75B108` 0 and the first player done
//! with the section (`menu::find_section_player`; none: nothing but a key taken). Then
//! the race's set-up from it (`finish_calls::set_up`), the sounds' and screens' set-up,
//! `drive::finish_race_0`, `dword_75B108` 0, the front end's set-up again (with the
//! race's sounds once more); a key taken at the end (eax 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The finders are their Rust versions (their fatal errors raised
//! as the original's); the calls through `part`, in the game through their addresses.
//! Ungated. Refuses the leader's ecx past 3 (the original reads past `g_players`) and what
//! the finders refuse, before anything. Shadow mode runs the original alone. Self-test: a
//! copy whose calls go to stand-ins, against the Rust version, in both event modes.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::finish_calls::{Game, PLACES, Part, game_part, regions};
use crate::menu::standings::*;
use crate::service::harness::{at, capture, restore};
use rc2k_core::menu::finish;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::championship_finish", rva(0x4A14CB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn championship_finish(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::finish::championship_finish`.
    finish::championship_finish(&mut Game { g: *entry, part })?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { championship_finish(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4A14CB,
        0x118,
        &[
            0x60, 0xC7, 0x05, 0x08, 0xB1, 0x75, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x29, 0x40, 0x6C, 0xB0, 0xB5, 0xBA, 0xBF, 0xC4, 0xC9, 0xCE, 0xD3, 0xE2, 0xE7, 0xEC,
            0xF1, 0xF6, 0xFB, 0x100, 0x105, 0x10A, 0x111,
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
    let table: u32 = unsafe { game_read(TABLE_B_RVA) };
    let Ok(all) = entries(table, B_ENTRY) else {
        return t.skip("no standings table B");
    };
    vars.push((table, all.len() * B_ENTRY as usize));
    let saved = capture(&vars);
    for case in 0..6u32 {
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
                game_write(rva(0x7556D0), if case % 2 == 1 { 0x13u32 } else { 0 });
                // a player entry with the group the finder wants, its +0x70 by lot
                if let Some(&e) = all.get(3) {
                    put(e + 4, 1 + case % 3);
                    put(e + 0x10, if case % 2 == 1 { 0 } else { 2 });
                    // the leader's path only outside event mode 0x13: there `first_player_b`'s ecx
                    // (computer drivers counted) could pass the players and the original would
                    // read past them
                    put(e + 0x70, u32::from(case < 3 && case % 2 == 0));
                }
            },
            &|r| unsafe { championship_finish(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
