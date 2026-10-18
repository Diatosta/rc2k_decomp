//! `race::hold_to_limit` (0x4650A9): eax held to `dword_71C6D4` (unsigned).
//!
//! In:  eax
//! Out: eax; the rest kept (the callers go on with a `mov`: no flags read)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45497E`, code at 0x454BD4, `sub_45C7EE`.
//! Shadow mode: by state (registers). Self-test: values below, at and above limits;
//! `dword_71C6D4` put back.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::hold_to_limit", rva(0x4650A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIMIT: usize = rva(0x71C6D4); // dword_71C6D4

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, hold_to_limit(r.eax)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = [at(LIMIT)];
    let saved = capture(&kept);
    for (limit, eax) in [
        (10u32, 3u32),
        (10, 10),
        (10, 11),
        (0, 5),
        (0xFFFF_FFFF, 0x8000_0000),
        (5, 0xFFFF_FFFF),
    ] {
        check_state(
            &mut t,
            &format!("limit {limit:#x} eax {eax:#x}"),
            call_original,
            Regs {
                eax,
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(LIMIT, limit) },
            &kept,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
