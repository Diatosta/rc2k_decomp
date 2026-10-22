//! `race::store_results_by_mode` (0x449957): the race's results kept by
//! `dword_62389C`: 2 the first block's +0x8C, +0x654 (also `dword_6051BC`) and +0x88;
//! 0 the two players' +0x88 and `g_stageCarTotal` x 100 less each (held to 0 ..
//! 300000); 1 and 3 the two +0x88s; others nothing.
//!
//! In:  nothing
//! Out: eax = the last value (mode 0: ebx the stage held to 0 .. 0x4C); the rest
//!      preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_4486FB` (a `mov` follows). Shadow mode:
//! the Rust version, the results put back, the original's, compared. Self-test: modes 0
//! .. 4, times round the bounds, the game's variables put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::store_results_by_mode",
    rva(0x449957),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    result_regions()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { store_results_by_mode(r) };
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
    let inputs = [
        0x62389Cusize,
        0x70F478,
        0x70FA40,
        0x70F474,
        0x710454,
        0x623850,
        0x601C4C,
    ];
    let mut kept = regions.clone();
    kept.extend(inputs.iter().map(|&a| at(rva(a))));
    let saved = capture(&kept);
    for (mode, times, stage, total) in [
        (0u32, [100u32, 200u32], 0x22u32, 50u32),
        (0, [0x8000_0000, 0], 0xFFFF_FFFF, 4000),
        (0, [10, 0xFFFF_FFF0], 0x60, 0x7FFF_FFFF),
        (1, [1, 2], 0, 0),
        (2, [3, 4], 0, 0),
        (3, [5, 6], 0, 0),
        (4, [7, 8], 0, 0),
    ] {
        check_state(
            &mut t,
            &format!("mode {mode}, times {times:x?}, stage {stage:#x}, total {total:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(rva(0x62389C), mode);
                game_write(rva(0x70F474), times[0]);
                game_write(rva(0x710454), times[1]);
                game_write(rva(0x70F478), 0x1111u32);
                game_write(rva(0x70FA40), 0x2222u32);
                game_write(rva(0x623850), stage);
                game_write(rva(0x601C4C), total);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
