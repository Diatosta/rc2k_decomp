//! `menu::number_all_a` (0x4A2090): the entries of standings table A numbered 1, 2, .. in
//! order (+0x44; with `dword_75AB40` 2 only those with a group, +0x1C not 0), a
//! player's number also into its record (+0x150); the count into `dword_75597C`.
//!
//! In:  nothing
//! Out: ecx the count, edi the end entry; esi / eax the last player's record and number
//!      less 1 (when one was numbered); the rest kept
//!
//! Callers: the event drivers. Refuses a table without its end entry or a player past 4
//! or missing, before anything. Self-test: the original and the Rust version on the
//! game's table and players, both modes; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::number_all_a", rva(0x4A2090), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MODE_RVA: usize = rva(0x75AB40); // dword_75AB40
const COUNT_RVA: usize = rva(0x75597C); // dword_75597C
const EXTRA: [usize; 1] = [MODE_RVA];
const CASES: u32 = 2;

fn prepare(case: u32) {
    let _ = case;
    unsafe { game_write(MODE_RVA, if case == 1 { 2u32 } else { 0 }) };
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn number_all_a(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    let grouped = unsafe { game_read::<u32>(MODE_RVA) } == 2;
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::{Numbering, number}`.
    use rc2k_core::menu::standings::{Numbering, number};
    let which = Numbering::All { grouped };
    let groups: Vec<u32> = all.iter().map(|&e| get(e + 0x1C)).collect();
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
            put(g.esi + 0x150, n);
        }
        put(e + 0x44, n);
    }
    g.edi = first + A_ENTRY * all.len() as u32;
    unsafe { game_write(COUNT_RVA, g.ecx) };
    Ok(g)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    let mut v: Vec<Region> = (0..4u32)
        .map(|n| (player(n), 0x400))
        .filter(|&(p, _)| p != 0)
        .collect();
    match entries(first, A_ENTRY) {
        Ok(all) => v.push((first, all.len() * A_ENTRY as usize)),
        Err(_) => v.push((0, 0)),
    }
    v.push(at(COUNT_RVA));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { number_all_a(r) }?))
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
    let before = Regs::SENTINEL;
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { number_all_a(&before) }.is_err();
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
