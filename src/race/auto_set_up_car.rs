//! `race::auto_set_up_car` (0x436F08): car block esi's set-up made for the stage (the
//! table `dword_6043FC` by `g_stageNumber`), with its log ("autosetup", "t", "off", "h",
//! "s", "g", "g" and the ranges).
//!
//! In:  esi = the car block
//! Out: eax = the last value (the stage when it has no set-up), ebx = 10, edx = the
//!      remainder, ebp = the set-up's address; the rest preserved
//!
//! See `race/race_cars.rs`. Callers: `race::set_up_mode_0_cars`, `sub_43613A` (mode 2).
//! Refuses stage 0x14 (the original's index wraps: a read far outside; section 16),
//! before anything. Shadow mode runs the original alone (it logs). Self-test: a copy whose
//! log calls note themselves, on the game's first block (filled, put back), stages round
//! the table's ends; the refusal on the Rust version alone.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::race_cars::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::auto_set_up_car", rva(0x436F08), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { auto_set_up_car(r, &mut cars_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { auto_set_up_car(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

const CALLS: &[usize] = &[
    0x31, 0x43, 0x4A, 0x7D, 0x84, 0xA0, 0xA7, 0xB4, 0xC7, 0xCE, 0xDB, 0xEF, 0xF6, 0x103, 0x116,
    0x11D, 0x12A, 0x130,
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x436F08,
        0x136,
        &[0xC7, 0x46, 0x20, 0x80, 0x00, 0x00, 0x00],
        CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = set_up_regions();
    let saved = capture(&regions);
    let block = game_addr(rva(BLOCK_0)) as u32;
    for stage in [0u32, 0x13, 0x15, 0x1E, 0x1F, 0x28, 0x33, 0x4F, 0x50, 0xFF] {
        run_cars(
            &mut t,
            &format!("stage {stage:#x}"),
            &copy,
            &regions,
            Regs {
                esi: block,
                ..Regs::SENTINEL
            },
            &|| {
                prepare(0x7777_0000 ^ stage, 0, 1, 1, [0, 0]);
                prepare_set_up(stage, 0, [0, 0]);
            },
            &|r| unsafe { auto_set_up_car(r, &mut test_part) },
        );
    }
    unsafe { game_write(STAGE_RVA, 0x14u32) };
    let refused = unsafe {
        auto_set_up_car(
            &Regs {
                esi: block,
                ..Regs::SENTINEL
            },
            &mut test_part,
        )
    }
    .is_err();
    t.check(refused, || "stage 0x14: not refused".to_string());
    restore(&regions, &saved);
    t.finish()
}
