//! `race::sort_checkpoints` (0x4458A7): the best line's checkpoint faces found and
//! chained in order.
//!
//! In:  nothing
//! Out: eax the last checkpoint (count - 1) with a chain; ebx, ecx, edx, esi, ebp as the
//!      searches leave them (see `race/best_line.rs`)
//!
//! Callers: `sub_445772` (the best line) and `sub_459BCA`. Refuses no faces or 0x100000 or
//! more, and more line faces than the list holds. Manual patch, private copy (a call at
//! the start). Self-test: faces of the test's (chains whole, broken, without a start, too
//! few), the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::sort_checkpoints", rva(0x4458A7), thunk, &ORIGINAL)
    .private_copy(0x138)
    .relocate_calls(&[0x0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        (game_addr(CHECK_FACES_RVA) as u32, 4 * MOST_CHECKS as usize),
        at(FOUND_RVA),
        at(CHECK_COUNT_RVA),
        at(LAST_CHECK_RVA),
        at(LINE_KIND_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        sort_checkpoints(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// A course of 40 faces: those at 3, 7, 11, .. (`line`) of the line's kind, chained by
/// +0xC from `start` (the one with +0x26 1) in the order `chain`, broken at `break_at`.
fn course(kind: u8, start: Option<usize>, break_at: usize) -> TestLine {
    let line: Vec<usize> = (0..9).map(|k| 3 + 4 * k).collect();
    let chain = [5usize, 2, 7, 0, 8, 1, 4, 6, 3];
    TestLine::new(40, 0, &|_| 0, &|n, f| {
        f.fill(0x11);
        if let Some(k) = line.iter().position(|&m| m == n) {
            f[0] = 0xFF;
            f[1] = kind;
            f[0x26] = if start == Some(k) { 1 } else { 2 };
            let at = chain.iter().position(|&c| c == k).unwrap();
            let next = if at + 1 < chain.len() && at + 1 != break_at {
                line[chain[at + 1]] as u32
            } else {
                0xDEAD
            };
            f[0xC..0x10].copy_from_slice(&next.to_le_bytes());
        }
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.push(at(DIRECTION_RVA));
    let saved = capture(&all);
    for (what, direction, kind, start, break_at) in [
        ("a whole chain", 0u32, 0x3Eu8, Some(5usize), 99usize),
        ("a broken chain", 0, 0x3E, Some(5), 4),
        ("a chain from its middle", 1, 0x3D, Some(0), 99),
        ("no start", 0, 0x3E, None, 99),
        ("no faces of the kind", 1, 0x3E, Some(5), 99),
    ] {
        let course = course(kind, start, break_at);
        let mut regions = regions();
        regions.push(course.faces.region());
        check_state(
            &mut t,
            what,
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
                game_write(DIRECTION_RVA, direction);
            },
            &regions,
            &rust,
        );
        drop(course);
    }
    restore(&all, &saved);
    t.finish()
}
