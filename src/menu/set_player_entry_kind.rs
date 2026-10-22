//! `menu::set_player_entry_kind` (0x4A1922): the current player's (`dword_75598C`) +0xD0
//! 1 when `dword_75AC80` is 100 or more (signed), else 2.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`. Refuses a current player past 3 or missing, before
//! anything. Self-test: the original and the Rust version on players of the test's.

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

pub static HOOK: RawHook = RawHook::new(
    "menu::set_player_entry_kind",
    rva(0x4A1922),
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

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const SCORE_RVA: usize = rva(0x75AC80); // dword_75AC80
const EXTRA: [usize; 2] = [CURRENT_RVA, SCORE_RVA];
const CASES: u32 = 6;
fn entry_of(_case: u32) -> Regs {
    Regs::SENTINEL
}
fn prepare(case: u32) {
    unsafe {
        game_write(CURRENT_RVA, case % 4);
        game_write(
            SCORE_RVA,
            [0u32, 99, 100, 250, 0xFFFF_FF00, 101][case as usize],
        );
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn set_player_entry_kind(entry: &Regs) -> Result<Regs, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::player::entry_kind`.
    let score: i32 = unsafe { game_read(SCORE_RVA) };
    put(player(n) + 0xD0, rc2k_core::menu::player::entry_kind(score));
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let r: Vec<Region> = (0..4u32)
        .map(|n| (player(n), 0x400))
        .filter(|&(p, _)| p != 0)
        .collect();
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { set_player_entry_kind(r) }?))
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
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    vars.push((game_addr(PLAYERS_RVA) as u32, 0x10));
    let saved = capture(&vars);
    let mut blocks = vec![0u32; 4 * 0x100];
    let bp = blocks.as_mut_ptr() as u32;
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            entry_of(case),
            &|| unsafe {
                restore(&vars, &saved);
                for n in 0..4u32 {
                    game_write(PLAYERS_RVA + 4 * n as usize, bp + 0x400 * n);
                    for o in (0..0x400).step_by(4) {
                        put(bp + 0x400 * n + o, (o * 3 + n * 11 + case * 5) & 0x1F);
                    }
                }
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    drop(blocks);
    t.finish()
}
