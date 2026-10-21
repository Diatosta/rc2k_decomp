//! `menu::find_section_player` (0x4A940E): the first of the four players whose entry for
//! the section (`section`, +0xB4 + 4 * section) is 0x14 (a reading: done with it); none:
//! ecx -1.
//!
//! In:  nothing (`section`)
//! Out: eax the section, ecx the player (or -1), edi the last player looked at
//!
//! Callers: `sub_4A14CB`, `sub_4A9439`. Refuses a missing player, before anything.
//! Self-test: the original and the Rust version on players of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::find_section_player", rva(0x4A940E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SECTION_RVA: usize = rva(0x75595C); // section
const EXTRA: [usize; 1] = [SECTION_RVA];
const CASES: u32 = 6;
fn entry_of(_case: u32) -> Regs {
    Regs::SENTINEL
}
fn prepare(case: u32) {
    unsafe { game_write(SECTION_RVA, case * 3) };
    if case < 5 {
        put(player(case % 4) + 0xB4 + 4 * case * 3, 0x14);
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn find_section_player(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    // Converted: `rc2k_core::menu::points::section_player`.
    g.eax = unsafe { game_read(SECTION_RVA) };
    let done: [u32; 4] = std::array::from_fn(|n| {
        get(player(n as u32)
            .wrapping_add(g.eax.wrapping_mul(4))
            .wrapping_add(0xB4))
    });
    let found = rc2k_core::menu::points::section_player(done);
    g.edi = player(found.unwrap_or(3));
    g.ecx = found.unwrap_or(u32::MAX);
    Ok(g)
}

fn regions() -> Vec<Region> {
    let r: Vec<Region> = (0..4u32)
        .map(|n| (player(n), 0x400))
        .filter(|&(p, _)| p != 0)
        .collect();
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { find_section_player(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
