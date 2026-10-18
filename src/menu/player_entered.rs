//! `menu::player_entered` (0x4A95EB): eax kept when player ecx's +0xD0 is 1 (a reading: the
//! player takes part), else eax 0.
//!
//! In:  ecx the player, eax a value
//! Out: edi the player, eax as above; the rest kept
//!
//! Caller: `sub_473126` (four times). Refuses a player past 3 or missing, before
//! anything. Self-test: the original and the Rust version on players of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::player_entered", rva(0x4A95EB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 8;
fn entry_of(case: u32) -> Regs {
    Regs {
        ecx: case % 4,
        eax: 0x1234 + case,
        ..Regs::SENTINEL
    }
}
fn prepare(case: u32) {
    put(player(case % 4) + 0xD0, case / 4);
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn player_entered(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.ecx >= 4 || player(g.ecx) == 0 {
        return Err(BadInput("a player past 3 or missing").into());
    }
    g.edi = player(g.ecx);
    if get(g.edi + 0xD0) != 1 {
        g.eax = 0;
    }
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
    Ok(left_of(&unsafe { player_entered(r) }?))
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
