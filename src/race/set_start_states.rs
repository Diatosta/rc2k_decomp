//! `race::set_start_states` (0x446D2E): the start values of the first car's kind and
//! of `dword_62385C` into `dword_6D14CA` / `CE` (the kind's record: +0x18 unless its
//! limit is past 0x1B58); `dword_6238A4` 0 or held to 1 .. 2 (two players); the
//! blocks' +0x684 2 or 3 made 4 where the kind is not flagged (`dword_5C5AC0`).
//!
//! In:  nothing
//! Out: eax = the second value, ebx = the last record offset (one player chosen: the
//!      chosen car's block); the rest preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_443F0B`, `sub_443F96`, code at 0x444005,
//! `sub_463666`, `sub_463A94` (none reads flags after it). Refuses, with
//! `dword_52A910` 1, a chosen car (`dword_52A920`) past 7, before anything. Shadow
//! mode: the Rust version, the values put back, the original's, compared. Self-test: one
//! player chosen and not, one and two players, kinds flagged and not and past the tables,
//! states 1 .. 4, the gear choice round its bounds, all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_start_states", rva(0x446D2E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    start_state_regions()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_start_states(r) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

#[allow(unused_variables)] // not every test uses every helper
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let _ = (
        game_addr(0),
        at(0),
        game_write::<u32> as unsafe fn(usize, u32),
    );
    let regions = regions();
    let blocks = crate::race::race_cars::saved_regions();
    let mut kept = regions.clone();
    kept.extend(blocks.iter().copied());
    kept.extend([at(rva(0x52A910)), at(rva(0x52A920)), at(rva(0x63DB94))]);
    let saved = capture(&kept);
    let block = |n: u32| -> u32 {
        unsafe {
            crate::framework::game_read(crate::race::objects::CAR_BLOCKS_RVA + 4 * n as usize)
        }
    };
    for (one, chosen, players, kinds, choice, states) in [
        (0u32, 0u32, 1u32, [0u32, 1u32], 5i32, [2u32, 3u32]),
        (0, 0, 2, [3, 0x30], -1, [3, 2]),
        (0, 0, 2, [0x21, 0x22], i32::MIN, [1, 4]),
        (1, 3, 1, [5, 7], 1, [2, 3]),
        (1, 7, 2, [0x40, 2], 2, [3, 3]),
        (2, 0, 2, [1, 1], 0x7FFF_FFFF, [2, 2]),
    ] {
        check_state(
            &mut t,
            &format!(
                "one {one}, car {chosen}, players {players}, kinds {kinds:x?}, choice {choice}, states {states:?}"
            ),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(rva(0x52A910), one);
                game_write(rva(0x52A920), chosen);
                game_write(rva(0x63DB94), players);
                game_write(GEAR_CHOICE_RVA, choice);
                for n in 0..8u32 {
                    game_write(
                        rva(0x623858) + 4 * n as usize,
                        kinds[(n & 1) as usize] + n / 2,
                    );
                    ((block(n) + 0x24) as *mut u32).write(kinds[(n & 1) as usize]);
                    ((block(n) + 0x684) as *mut u32).write(states[(n & 1) as usize]);
                }
                game_write(rva(0x62385C), kinds[1]);
            },
            &regions,
            &rust,
        );
    }
    unsafe {
        game_write(rva(0x52A910), 1u32);
        game_write(rva(0x52A920), 8u32);
    }
    let refused = unsafe { set_start_states(&Regs::SENTINEL) }.is_err();
    t.check(refused, || "chosen car 8: not refused".to_string());
    restore(&kept, &saved);
    t.finish()
}
