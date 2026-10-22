//! `race::set_up_arcade_cars` (0x4468FB): with `dword_62389C` 2, "SETUPARCADECARS" logged,
//! `race::choose_arcade_opponents` with the log indented, `g_raceCars` 4; then each
//! block's +0x24 its kind.
//!
//! In:  nothing
//! Out: the registers as `choose_arcade_opponents` leaves them
//!
//! See `race/race_cars.rs`. Callers: `sub_443F0B`, `sub_443F96`, code at 0x444000,
//! `race::credits_race`, `sub_463A94` (`race::set_start_states` follows each; no flags read).
//! Shadow mode runs the original alone (it logs). Self-test: a copy whose two calls go to
//! stand-ins that note them, its two references to `g_logIndent` the test's, against the
//! Rust version on the same, modes 2 and 0.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::race_cars::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_up_arcade_cars", rva(0x4468FB), thunk, &ORIGINAL).self_test(self_test);

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
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    match unsafe { set_up_arcade_cars(r, indent, &mut cars_part) } {
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
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    {
        let g = unsafe { set_up_arcade_cars(r, indent, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let indent = Box::new(0u32);
    let at = &*indent as *const u32 as u32;
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x4468FB,
            len: 0x98,
            entry: &[0x83, 0x3D, 0x9C, 0x38, 0x62, 0x00, 0x02],
            calls: &[0x10, 0x1F],
            jumps_out: &[],
        },
        &|c| {
            CAR_CALLEES
                .iter()
                .find(|&&(k, _)| rva(k) == c)
                .map(|&(_, p)| crate::install::stand_ins::stand_in(p))
        },
        &[(0x4E00A0, at)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = opponent_regions();
    regions.push((at, 4));
    let saved = capture(&regions);
    for mode in [2u32, 0] {
        run_cars(
            &mut t,
            &format!("mode {mode}"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(MODE_RVA, mode);
                for n in 0..8 {
                    game_write(CAR_KINDS_RVA + 4 * n, 0x30u32 + n as u32);
                }
                (at as *mut u32).write(0x30);
            },
            &|r| unsafe { set_up_arcade_cars(r, at, &mut crate::race::mud::test_part) },
        );
    }
    restore(&regions, &saved);
    drop(indent);
    t.finish()
}
