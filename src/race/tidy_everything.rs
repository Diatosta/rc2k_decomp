//! `race::tidy_everything` (0x444AC7): the race tidied ("SHAUNTIDYEVERYTHING"): the
//! controls, the game mode, the stage (`race::tidy_stage`), the sounds; `dword_63DB74`
//! and `dword_63DB78` 1; each step logged with the log indented round it.
//!
//! In:  nothing
//! Out: the registers as the steps leave them
//!
//! See `race/race_end.rs`. Seven callers (`sub_443F0B`, `sub_443F96`, menus; none reads
//! flags after it). It begins with its call: private copy, its 14 calls relocated, manual
//! patch. A step that refuses runs in the game through its hook. Shadow mode runs the
//! original alone (it logs and frees). Self-test: a copy whose 14 calls go to stand-ins,
//! its 10 references to `g_logIndent` the test's, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::loadlevel::LoadVars;
use crate::race::race_end::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::tidy_everything", rva(0x444AC7), thunk, &ORIGINAL)
    .private_copy(0xF5)
    .relocate_calls(&[
        0x0, 0xC, 0x19, 0x38, 0x47, 0x5C, 0x6B, 0x80, 0x8F, 0xA4, 0xB3, 0xDC, 0xE9, 0xEF,
    ])
    .manual_patch()
    .self_test(self_test);

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
    match unsafe { tidy_everything(r, &LoadVars::game(), &mut tidy_part) } {
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
        let g = unsafe { tidy_everything(r, &LoadVars::game(), &mut tidy_part) }?;
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
    let vars = Box::new([0u32; 3]);
    let base = vars.as_ptr() as u32;
    let test = LoadVars {
        indent: base,
        log_enabled: base + 4,
        in_race: base + 8,
        bar: 0,
    };
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x444AC7,
            len: 0xF5,
            entry: &[0xE8, 0x12, 0xC9, 0xFB, 0xFF],
            calls: &[
                0x0, 0xC, 0x19, 0x38, 0x47, 0x5C, 0x6B, 0x80, 0x8F, 0xA4, 0xB3, 0xDC, 0xE9, 0xEF,
            ],
            jumps_out: &[],
        },
        &|c| {
            TIDY_CALLEES
                .iter()
                .position(|&k| k != 0 && rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        },
        &[(0x4E00A0, test.indent)],
        10,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = tidy_regions();
    let saved = capture(&regions);
    regions.push((base, 12));
    for fill in [0u32, 0x5A5A_5A5A] {
        crate::race::mud::run_both(
            &mut t,
            &format!("filled {fill:#x}"),
            &copy,
            &regions,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, fill as u8, len);
                }
                let v = base as *mut u32;
                v.write(0x30);
                v.add(1).write(1);
                v.add(2).write(1);
            },
            &|r| unsafe { tidy_everything(r, &test, &mut crate::race::mud::test_part) },
        );
    }
    let _ = game_addr(0);
    restore(&tidy_regions(), &saved);
    drop(vars);
    t.finish()
}
