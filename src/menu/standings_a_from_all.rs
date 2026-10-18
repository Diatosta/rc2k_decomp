//! `menu::standings_a_from_all` (0x4A2B75): standings table A filled from the drivers
//! (`menu::standings_from_drivers`: emptied, each driver with its name), then each group's
//! (+0x90, 0 .. 3) counted players into the entries from 89, 139, 169 and 179 down
//! (`menu::copy_standing_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A94EC`. The steps are their Rust versions. Refuses a
//! player count of 0 or past 4, a player missing or without a name, and what the drivers'
//! fill refuses, before anything. Self-test: the original and the Rust version on the
//! game's tables, the players' groups by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::standings_a_from_all",
    rva(0x4A2B75),
    thunk,
    &ORIGINAL,
)
.private_copy(0x15B)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

/// Its calls (the first in the patched bytes).
const CALLS: [usize; 7] = [0x1, 0x55, 0x5A, 0x97, 0xD1, 0x10B, 0x145];

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 1] = [PLAYER_COUNT_RVA];
const CASES: u32 = 4;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`copy_standing_entry`, ebx the player's index).
unsafe fn players_by_group(
    entry: &Regs,
    first: u32,
    size: u32,
    slots: [u32; 4],
) -> Result<(), Refused> {
    let players = counted_players()?;
    for (group, slot) in slots.into_iter().enumerate() {
        let mut edi = first + slot;
        for (n, &p) in players.iter().enumerate() {
            if get(p + 0x90) == group as u32 {
                unsafe {
                    crate::menu::copy_standing_entry::copy_standing_entry(&Regs {
                        edi,
                        esi: p,
                        ebx: n as u32,
                        ..*entry
                    })
                }?;
                edi -= size;
            }
        }
    }
    Ok(())
}

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    unsafe { game_write(PLAYER_COUNT_RVA, 1 + case) };
    for n in 0..4u32 {
        if player(n) != 0 {
            put(player(n) + 0x90, (n * 3 + case) % 4);
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn standings_a_from_all(entry: &Regs) -> Result<Regs, Refused> {
    let players = counted_players()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    unsafe { crate::menu::standings_from_drivers::standings_from_drivers(entry) }?;
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    unsafe { players_by_group(entry, first, A_ENTRY, [0x363C, 0x54B4, 0x66FC, 0x6D14]) }?;
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let v = vec![(unsafe { game_read(TABLE_A_RVA) }, (200 * A_ENTRY) as usize)];
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { standings_a_from_all(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    // `prepare` writes the players' +0x90: put back too (left changed until 2026-10-18).
    vars.extend(
        (0..4u32)
            .map(player)
            .filter(|&p| p != 0)
            .map(|p| (p + 0x90, 4)),
    );
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { standings_a_from_all(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(case),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
