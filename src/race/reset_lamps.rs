//! `race::reset_lamps` (0x45487A): `g_lampCount`, `dword_63DC4C`, `dword_63DC50` (the
//! sort entries' count) and `dword_64158C` 0, then `race::object_bytes`.
//!
//! In:  nothing
//! Out: as `race::object_bytes` leaves them (a `retn` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455803), each view drawn. Shadow
//! mode: by state (the four dwords and the four 0x100-byte tables). Self-test: on the
//! game's tables (the four dwords set first); all put back (skipped without view blocks,
//! as the tables are the stage's).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_lamps", rva(0x45487A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut v: Vec<Region> = [LAMP_COUNT_RVA, rva(0x63DC4C), SORT_COUNT_RVA, rva(0x64158C)]
        .iter()
        .map(|&a| at(a))
        .collect();
    v.extend(object_tables()[5..].iter().map(|&a| (a, 0x100)));
    v
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_lamps() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if VIEW_BLOCKS
        .iter()
        .any(|&a| unsafe { game_read::<u32>(a) } == 0)
    {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "the game's tables",
        call_original,
        Regs::SENTINEL,
        &|| unsafe {
            for &(a, len) in &regions {
                std::ptr::write_bytes(a as *mut u8, 0xAA, len);
            }
            game_write(LAMP_COUNT_RVA, 7u32);
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
