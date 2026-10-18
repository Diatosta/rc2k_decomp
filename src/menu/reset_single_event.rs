//! `menu::reset_single_event` (0x4AAA2F): `dword_755254` / `dword_755950` 0; every
//! driver's +0x10 / +0xC / +0x20 0 and +0x24 1; `dword_75592C` 1, `dword_755978` 0; the
//! four players' +0x8C / +0x84 0 and +0xD0 1 (a reading: a single event's start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4AA4D7` (twice). Refuses a missing player or a drivers' table without its
//! end entry, before anything. Self-test: the original and the Rust version on the game's
//! tables; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::reset_single_event", rva(0x4AAA2F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 2;

fn prepare(case: u32) {
    let _ = case;
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_single_event(entry: &Regs) -> Result<Regs, Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    unsafe {
        game_write(rva(0x755254), 0u32);
        game_write(rva(0x755950), 0u32);
    }
    for e in drivers {
        put(e + 0x10, 0);
        put(e + 0xC, 0);
        put(e + 0x20, 0);
        put(e + 0x24, 1);
    }
    unsafe {
        game_write(rva(0x75592C), 1u32);
        game_write(rva(0x755978), 0u32);
    }
    for n in (0..4).rev() {
        put(player(n) + 0x8C, 0);
        put(player(n) + 0x84, 0);
        put(player(n) + 0xD0, 1);
    }
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    let mut v: Vec<Region> = (0..4u32).map(|n| (player(n), 0x400)).collect();
    let first = game_addr(DRIVERS_RVA) as u32;
    match entries(first, DRIVER) {
        Ok(all) => v.push((first, all.len() * DRIVER as usize)),
        Err(_) => v.push((0, 0)),
    }
    v.extend([rva(0x755254), rva(0x755950), rva(0x75592C), rva(0x755978)].map(at));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_single_event(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = Regs::SENTINEL;
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { reset_single_event(&before) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            before,
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
