//! `race::tidy_stage` (0x444940): the stage unloaded ("LOADLEVEL_TIDY"): the mud, the
//! point records, the marks' tidy (`marks::tidy_marks`, with the log off), the
//! fonts, the map, the textures, the course's two memories, the objects; `g_inRace` 0,
//! `dword_606294` `dword_6235F4`; each step logged with the log indented round it.
//!
//! In:  nothing
//! Out: the registers as the steps leave them
//!
//! See `race/race_end.rs`. One caller, `race::tidy_everything`. Its steps are the Rust
//! versions; a step that refuses runs in the game through
//! its hook. Shadow mode runs the original alone (it logs and frees). Self-test: a copy
//! whose 19 calls go to stand-ins, its 22 references to `g_logIndent`, `g_logEnabled`
//! and `g_inRace` the test's, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::loadlevel::LoadVars;
use crate::race::race_end::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::tidy_stage", rva(0x444940), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { tidy_stage(r, &LoadVars::game(), &mut tidy_part) } {
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
        let g = unsafe { tidy_stage(r, &LoadVars::game(), &mut tidy_part) }?;
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
            address: 0x444940,
            len: 0x187,
            entry: &[0x60, 0x8D, 0x05, 0xD7, 0xC1, 0x4D, 0x00],
            calls: &[
                0x7, 0x16, 0x2B, 0x3A, 0x4F, 0x6E, 0x89, 0x98, 0xAD, 0xBC, 0xDA, 0xE9, 0xFE, 0x10D,
                0x122, 0x131, 0x146, 0x155, 0x180,
            ],
            jumps_out: &[],
        },
        &|c| {
            TIDY_CALLEES
                .iter()
                .position(|&k| k != 0 && rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        },
        &[
            (0x4E00A0, test.indent),
            (0x4E0094, test.log_enabled),
            (0x5C5CEC, test.in_race),
        ],
        22,
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
            &|r| unsafe { tidy_stage(r, &test, &mut crate::race::mud::test_part) },
        );
    }
    let _ = game_addr(0);
    restore(&tidy_regions(), &saved);
    drop(vars);
    t.finish()
}
