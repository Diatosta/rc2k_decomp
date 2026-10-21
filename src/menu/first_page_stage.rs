//! `menu::first_page_stage` (0x4A2440): the first stage on the last page (`dword_7556B8`)
//! for `dword_7556B4` stages, ten a page: 1 up to ten, else (n / 10) * 10 + 1, less ten when
//! n is a multiple of ten; n 0 there: the fatal error 0x80701010 (cannot happen past the
//! first test).
//!
//! In:  nothing
//! Out: unchanged up to ten; else eax the first stage, edx n % 10, ebp 10
//!
//! Callers: the event pages' code. Self-test: the original and the Rust version, counts
//! by lot.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::first_page_stage", rva(0x4A2440), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
const FIRST_RVA: usize = rva(0x7556B8); // dword_7556B8
const EXTRA: [usize; 1] = [COUNT_RVA];
const CASES: u32 = 7;

fn test_entry() -> Regs {
    Regs::SENTINEL
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
    unsafe {
        game_write(
            COUNT_RVA,
            [0u32, 3, 10, 11, 20, 37, 0xFFFF_FFF0][case as usize],
        )
    };
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn first_page_stage(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n: u32 = unsafe { game_read(COUNT_RVA) };
    // Converted: `rc2k_core::menu::event::last_page` (its division cannot meet 0).
    let Some(last) = rc2k_core::menu::event::last_page(n) else {
        unsafe { game_write(FIRST_RVA, 1u32) };
        return Ok(g);
    };
    g.edx = last.rest;
    g.eax = last.first;
    g.ebp = 10;
    unsafe { game_write(FIRST_RVA, g.eax) };
    Ok(g)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![at(FIRST_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { first_page_stage(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = test_entry();
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { first_page_stage(&before) }.is_err();
    restore(&vars, &saved);
    if refused && !refusal_expected() {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(),
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
