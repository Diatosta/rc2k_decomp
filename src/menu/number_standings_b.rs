//! `menu::number_standings_b` (0x4A4A9D): the entries of group ebx in standings table B numbered
//! 1, 2, .. in order (+0x70), a player's number also into its record (+0x144).
//!
//! In:  ebx the group
//! Out: ecx the group's count, edi the end entry; esi / eax the last player's record and
//!      number less 1 (when one was numbered); the rest kept
//!
//! Callers: the event pages' code. Refuses a table without its end entry or a player past
//! 4 or missing, before anything. Self-test: the original and the Rust version on the
//! game's table and players, groups 1 .. 3; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::number_standings_b", rva(0x4A4A9D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 3;

fn test_entry() -> Regs {
    Regs {
        ebx: 1,
        ..Regs::SENTINEL
    }
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
    let _ = case;
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn number_standings_b(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    let all = entries(first, B_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::{Numbering, number};
    let which = Numbering::Group(g.ebx);
    let groups: Vec<u32> = all.iter().map(|&e| get(e + 0x10)).collect();
    for (&e, &group) in all.iter().zip(&groups) {
        let p = get(e + 4);
        if which.counts(group) && p != 0 && (p > 4 || player(p - 1) == 0) {
            return Err(BadInput("a standings player past 4 or missing").into());
        }
    }
    g.ecx = 0;
    for (&e, n) in all.iter().zip(number(&groups, which)) {
        let Some(n) = n else {
            continue;
        };
        g.ecx = n;
        if get(e + 4) != 0 {
            g.eax = get(e + 4) - 1;
            g.esi = player(g.eax);
            put(g.esi + 0x144, n);
        }
        put(e + 0x70, n);
    }
    g.edi = first + B_ENTRY * all.len() as u32;
    Ok(g)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    let mut v: Vec<Region> = (0..4u32)
        .map(|n| (player(n), 0x400))
        .filter(|&(p, _)| p != 0)
        .collect();
    if let Ok(all) = entries(first, B_ENTRY) {
        v.push((first, all.len() * B_ENTRY as usize));
    }
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { number_standings_b(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
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
    let before = test_entry();
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { number_standings_b(&before) }.is_err();
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
