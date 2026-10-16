//! `race::copy_course_header` (0x445736): the course memory's first 0x100 bytes to
//! `g_courseHeader`.
//!
//! In:  nothing
//! Out: esi and edi past the bytes, ecx 0; the rest preserved
//!
//! See `race/course_data.rs`. Callers `sub_4455FC` and `sub_459B6A`'s function. Self-test: a
//! course memory of the test's, the header put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::course_data::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_course_header", rva(0x445736), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(HEADER_RVA) as u32, HEADER_LEN)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        copy_course_header(r)
    }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.push(at(COURSE_MEMORY_RVA));
    let saved = capture(&all);
    for seed in [3u8, 0x5A] {
        let memory: Vec<u8> = (0..0x180u32)
            .map(|n| (n as u8).wrapping_mul(seed) ^ 0x33)
            .collect();
        check_state(
            &mut t,
            &format!("seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
                game_write(COURSE_MEMORY_RVA, memory.as_ptr() as u32);
            },
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
