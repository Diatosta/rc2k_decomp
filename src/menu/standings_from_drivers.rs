//! `menu::standings_from_drivers` (0x4A9605): standings table A emptied (`menu::init_standings_a`) and filled from the drivers in order: +0x14 / +0x1C / +0x18 / +8 / +0x10 / +0 / +0x4C from the driver's +0xC / +8 / +0x10 / +0x14 / +0x18 / +0 / +0x24, +4 0, +0xC the driver's name string (`g_driverNames`, 0x14 each, by its +0) and +0x20 its text.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. `strings::append_string_by_id`'s Rust version. Refuses no table, more drivers than its 200 entries, a name past the entry, before anything. Self-test: the original and the Rust version on the game's
//! tables; put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::standings_from_drivers",
    rva(0x4A9605),
    thunk,
    &ORIGINAL,
)
.private_copy(0x73)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

/// Its calls (the first in the patched bytes).
const CALLS: [usize; 3] = [0x1, 0x55, 0x5A];

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const NAMES_RVA: usize = rva(0x741E88); // g_driverNames

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn standings_from_drivers(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    if first == 0 || drivers.len() > 200 {
        return Err(BadInput("no standings table, or more drivers than it holds").into());
    }
    let mut texts = Vec::new();
    for &d in &drivers {
        let id: u32 = unsafe { game_read(NAMES_RVA + 0x14 * get(d) as usize) };
        let mut text = Vec::new();
        unsafe { crate::strings::append_string_by_id::push_string(&mut text, id) };
        text.push(0);
        if text.len() > (A_ENTRY - 0x20) as usize {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push((id, text));
    }
    unsafe { crate::menu::init_standings_a::init_standings_a(entry) }?;
    // Converted: `rc2k_core::menu::standings::fill_a_from_driver`.
    let count = drivers.len() as u32;
    let all = unsafe { table(drivers[0], DRIVER, count) };
    let mut t = unsafe { table(first, A_ENTRY, count) };
    for (n, (id, text)) in texts.into_iter().enumerate() {
        rc2k_core::menu::standings::fill_a_from_driver(&mut t, n, &all, n, id, &text);
    }
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    vec![(unsafe { game_read(TABLE_A_RVA) }, (200 * A_ENTRY) as usize)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { standings_from_drivers(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
    vars.push(at(rva(0x75598C)));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { standings_from_drivers(&Regs::SENTINEL) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..3u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&vars, &saved);
                let _ = case;
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
