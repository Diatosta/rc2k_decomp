//! `menu::standings_b_from_all` (0x4A44A0): standings table B emptied
//! (`menu::init_standings_b`) and filled from the drivers in order: +4 0, +0x10 / +8 /
//! +0xC / +0 / +0x34 the driver's +8 / +0x14 / +0x18 / +0 / +0x10, +0x38 .. +0x6C its
//! +0x5C .. +0x90, +0x14 the text of its name (`g_driverNames`, 0x14 each, by its +0);
//! then each group's (+0x90, 0 .. 3) counted players into the entries from 89, 119, 169
//! and 179 down (`menu::player_to_standing_b`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A9583`. The steps are their Rust versions
//! (`strings::append_string_by_id`'s for the names). Refuses no table, more drivers than
//! its 200 entries, a name past the entry, a player count of 0 or past 4, a player missing
//! or without a name, before anything. Self-test: the original and the Rust version on the
//! game's tables, the players' groups by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::standings_b_from_all",
    rva(0x4A44A0),
    thunk,
    &ORIGINAL,
)
.private_copy(0x1A4)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

/// Its calls (the first in the patched bytes).
const CALLS: [usize; 7] = [0x1, 0xA9, 0xAE, 0xEC, 0x123, 0x15A, 0x191];

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const NAMES_RVA: usize = rva(0x741E88); // g_driverNames
const EXTRA: [usize; 1] = [PLAYER_COUNT_RVA];
const CASES: u32 = 4;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`player_to_standing_b`, ebx the player's index).
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
                    crate::menu::player_to_standing_b::player_to_standing_b(&Regs {
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
pub unsafe fn standings_b_from_all(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    if first == 0 || drivers.len() > 200 {
        return Err(BadInput("no standings table, or more drivers than it holds").into());
    }
    let players = counted_players()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    let mut texts = Vec::new();
    for &d in &drivers {
        let id: u32 = unsafe { game_read(NAMES_RVA + 0x14 * get(d) as usize) };
        let mut text = Vec::new();
        unsafe { crate::strings::append_string_by_id::push_string(&mut text, id) };
        text.push(0);
        if text.len() > (B_ENTRY - 0x14) as usize {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push(text);
    }
    unsafe { crate::menu::init_standings_b::init_standings_b(entry) }?;
    for (n, (&d, text)) in drivers.iter().zip(texts).enumerate() {
        let e = first + B_ENTRY * n as u32;
        put(e + 4, 0);
        put(e + 0x10, get(d + 8));
        put(e + 8, get(d + 0x14));
        put(e + 0xC, get(d + 0x18));
        put(e, get(d));
        put(e + 0x34, get(d + 0x10));
        for k in 0..14 {
            put(e + 0x38 + 4 * k, get(d + 0x5C + 4 * k));
        }
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), (e + 0x14) as *mut u8, text.len()) };
    }
    unsafe { players_by_group(entry, first, B_ENTRY, [0x29B8, 0x37C8, 0x4F38, 0x53E8]) }?;
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    vec![(unsafe { game_read(TABLE_B_RVA) }, (200 * B_ENTRY) as usize)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { standings_b_from_all(r) }?))
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
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { standings_b_from_all(&test_entry(0)) }.is_err();
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
