//! `menu::inspection_verdict` (0x4A1952): the current player's (`dword_75598C`) inspection
//! verdict. The penalties (`dword_75AC48` ..) of its inspected parts at or past their
//! limits whose flag in `dword_759B6E` (index + 2) is 0 added into `dword_7559EC`; 100
//! less that into `dword_75AC80`; the player's +0xD0 1 when that is at least 100 (no
//! penalty), else 2.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A8556`. Refuses a current player past 3 or missing, before anything.
//! Self-test: the original and the Rust version on the game's player, parts around their
//! limits and flags by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::inspection_verdict", rva(0x4A1952), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FLAGS_RVA: usize = rva(0x759B6E); // dword_759B6E
const SUM_RVA: usize = rva(0x7559EC); // dword_7559EC
const SCORE_RVA: usize = rva(0x75AC80); // dword_75AC80
const EXTRA: [usize; 1] = [CURRENT_RVA];
const FLAG_COUNT: usize = 16;
const CASES: u32 = 4;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n < 4 && player(n) != 0 {
        for k in 0..14u32 {
            let limit: u32 = unsafe { game_read(LIMITS_RVA + 4 * k as usize) };
            let d = [0u32, 1, 0xFFFF_FFFF, 0x100][((k + case) % 4) as usize];
            put(player(n) + 0xD4 + 4 * k, limit.wrapping_add(d));
        }
    }
    for k in 0..16u32 {
        unsafe {
            game_write(
                FLAGS_RVA + 4 * k as usize,
                u32::from((k + case).is_multiple_of(3)),
            )
        };
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn inspection_verdict(entry: &Regs) -> Result<Regs, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::inspection::verdict`.
    let p = player(n);
    unsafe {
        let v = rc2k_core::menu::inspection::verdict(
            |k| past_limit(p, k as u32),
            |k| game_read::<u32>(FLAGS_RVA + 4 * (k + 2)) != 0,
            |k| game_read(PENALTIES_RVA + 4 * k),
        );
        game_write(SUM_RVA, v.penalties);
        game_write(SCORE_RVA, v.score);
        put(p + 0xD0, v.kind);
    }
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    let p = if n < 4 { player(n) } else { 0 };
    vec![
        at(SUM_RVA),
        at(SCORE_RVA),
        (p, 0x110),
        (game_addr(FLAGS_RVA) as u32, 4 * FLAG_COUNT),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { inspection_verdict(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { inspection_verdict(&test_entry(0)) }.is_err();
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
