//! `menu::copy_standings_summary` (0x470466): 0x5A entries of standings table A copied, in
//! order, into the summary table (`dword_7556F4`, 0x50 each: +0 / +4 / +8 / +0xC from the
//! entry's +0xC / +8 / +0x10 / +0x14): with `dword_75AB40` 2 the entries with a group
//! (+0x1C not 0), else those without.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. Refuses null tables and a table A that runs out (the original
//! walks on past its end), before anything. Self-test: the original and the Rust version
//! on the game's tables, both modes; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::copy_standings_summary",
    rva(0x470466),
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

const SUMMARY_RVA: usize = rva(0x7556F4); // dword_7556F4
const MODE_RVA: usize = rva(0x75AB40); // dword_75AB40
const COPIED: u32 = 0x5A;
const EXTRA: [usize; 1] = [MODE_RVA];
const CASES: u32 = 2;

fn prepare(case: u32) {
    let _ = case;
    unsafe { game_write(MODE_RVA, if case == 1 { 2u32 } else { 0 }) };
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn copy_standings_summary(entry: &Regs) -> Result<Regs, Refused> {
    let (summary, first): (u32, u32) = unsafe { (game_read(SUMMARY_RVA), game_read(TABLE_A_RVA)) };
    if summary == 0 || first == 0 {
        return Err(BadInput("no standings tables").into());
    }
    let grouped = unsafe { game_read::<u32>(MODE_RVA) } == 2;
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::summary_rows`.
    let t = unsafe { table(first, A_ENTRY, all.len() as u32) };
    let Some(rows) = rc2k_core::menu::standings::summary_rows(&t, grouped) else {
        return Err(BadInput("standings table A runs out").into());
    };
    for (n, row) in rows.into_iter().enumerate() {
        let s = summary + 0x50 * n as u32;
        for (k, v) in row.into_iter().enumerate() {
            put(s + 4 * k as u32, v);
        }
    }
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![(unsafe { game_read(SUMMARY_RVA) }, (0x50 * COPIED) as usize)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { copy_standings_summary(r) }?))
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
    let refused = unsafe { copy_standings_summary(&before) }.is_err();
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
