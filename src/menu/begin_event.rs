//! `menu::begin_event` (0x472BAA): the drivers and the four players made ready for a new
//! event.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755254` and `dword_755950` 0; every driver of `g_drivers` (0x94 bytes each, up to
//! the one whose +0 is -1; the first always) gets +0x10, +0xC, +0x20 = 0 and +0x24 = 1;
//! `dword_75592C` 1, `dword_755978` 0; then for each player n (the current player
//! `dword_75598C` = n, left at 3): `field_8c` and `field_84` 0, +0xD0 1, `service::repair_car`,
//! and unless the event mode `dword_7556D0` is 3 the setup's defaults: gear ratio 2, brake
//! balance 0x32, tyre 3, ride height 0x46, suspension 0x1E, steering sensitivity = the
//! controller's (+0x280). Callers: the event loops' starts (`sub_482382` and nine more).
//! Refuses drivers without their end within 0x400 (the original runs on), a missing player.
//! Self-test: the game's drivers and players, modes 0 and 3, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::player::{PLAYERS, player_at, player_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::begin_event", rva(0x472BAA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const DRIVERS_RVA: usize = rva(0x73B674); // g_drivers
pub const DRIVER_LEN: u32 = 0x94;
const MOST_DRIVERS: u32 = 0x400;
const CLEARED_A_RVA: usize = rva(0x755254); // dword_755254
const STATE_RVA: usize = rva(0x755950); // dword_755950
const NEW_EVENT_RVA: usize = rva(0x75592C); // dword_75592C
const CLEARED_B_RVA: usize = rva(0x755978); // dword_755978
const CURRENT_PLAYER_RVA: usize = crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
pub const MODE_RVA: usize = rva(0x7556D0); // dword_7556D0: the event's mode
/// The mode that keeps the players' setups.
const KEEPS_SETUP: u32 = 3;

/// The drivers' count (with the first), or `None` without their end.
pub fn drivers() -> Option<u32> {
    let first = game_addr(DRIVERS_RVA) as u32;
    (1..MOST_DRIVERS)
        .find(|&n| unsafe { ((first + n * DRIVER_LEN) as *const u32).read() } == u32::MAX)
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

pub unsafe fn begin_event() -> Result<(), Refused> {
    let count = drivers().ok_or(BadInput("the drivers do not end within 0x400"))?;
    if (0..PLAYERS).any(|n| player_at(n) == 0) {
        return Err(BadInput("a player missing from g_players").into());
    }
    unsafe {
        game_write(CLEARED_A_RVA, 0u32);
        game_write(STATE_RVA, 0u32);
        let first = game_addr(DRIVERS_RVA) as u32;
        for n in 0..count {
            let d = first + n * DRIVER_LEN;
            put(d + 0x10, 0);
            put(d + 0xC, 0);
            put(d + 0x20, 0);
            put(d + 0x24, 1);
        }
        game_write(NEW_EVENT_RVA, 1u32);
        game_write(CLEARED_B_RVA, 0u32);
        for n in 0..PLAYERS {
            game_write(CURRENT_PLAYER_RVA, n);
            let p = player_at(n);
            put(p + 0x8C, 0);
            put(p + 0x84, 0);
            put(p + 0xD0, 1);
            crate::service::repair_parts::repair_car()?;
            if game_read::<u32>(MODE_RVA) != KEEPS_SETUP {
                put(p + 0x120, 2);
                put(p + 0x110, 0x32);
                put(p + 0x10C, 3);
                put(p + 0x114, 0x46);
                put(p + 0x118, 0x1E);
                put(p + 0x11C, ((p + 0x280) as *const u32).read_unaligned());
            }
        }
    }
    Ok(())
}

fn regions() -> Vec<Region> {
    let mut r = player_regions();
    r.extend([
        at(CLEARED_A_RVA),
        at(STATE_RVA),
        at(NEW_EVENT_RVA),
        at(CLEARED_B_RVA),
        at(CURRENT_PLAYER_RVA),
    ]);
    if let Some(count) = drivers() {
        r.push((game_addr(DRIVERS_RVA) as u32, (count * DRIVER_LEN) as usize));
    }
    r
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_event() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if drivers().is_none() {
        return t.skip("the drivers do not end within 0x400");
    }
    if player_regions().len() != PLAYERS as usize {
        return t.skip("a player missing from g_players");
    }
    let regions = regions();
    let mut all = regions.clone();
    all.push(at(MODE_RVA));
    let saved = capture(&all);
    for mode in [0u32, KEEPS_SETUP, 0x13] {
        check_state(
            &mut t,
            &format!("mode {mode:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
                game_write(MODE_RVA, mode);
            },
            &regions,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
