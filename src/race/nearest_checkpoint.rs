//! `race::nearest_checkpoint` (0x44134F): the checkpoint nearest the work car (x
//! `dword_71A10C`, z `dword_71A114`, put in `dword_639824` / `dword_63982C`), by
//! `race::nearest_checkpoint_in` over them all; its distance in `dword_62400C` and / 65536
//! (`fild` / `fdiv` / `fstp`) in `g_nearestCheckpointDistance` (0x614C68).
//!
//! In:  nothing
//! Out: eax = the checkpoint (0 with none, then nothing else changed); else ecx and edx
//!      the count, ebp the checkpoint, edi its distance, ebx and esi as the last measure
//!      leaves them
//!
//! See `race/start_place.rs`. Callers: `sub_4464AD` (race init's camera start), `sub_43FB00`
//! and `sub_4413F2` (later). Refuses more checkpoints than the lists hold, before anything.
//! Shadow mode: the Rust version, the variables put back, the original's, compared.
//! Self-test: faces, points and checkpoints of the test's (`TestLine`), the car's place
//! set, the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::best_line::{CHECK_COUNT_RVA, TestLine};
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::nearest_checkpoint", rva(0x44134F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { nearest_checkpoint(r) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &nearest_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = nearest_regions();
    let mut kept = regions.clone();
    kept.extend([at(CAR_X_RVA), at(CAR_Z_RVA)]);
    let saved = capture(&kept);
    let line = TestLine::new(32, 30, &|k| (k * 7 + 1) % 32, &|_, f| f.fill(0x3C));
    for (count, car) in [
        (30u32, (0i32, 0i32)),
        (30, (0x12_3456, -0x7_0000)),
        (30, (-0x40_0000, 0x1_5555)),
        (30, (0x7000_0000, 0x7000_0000)),
        (1, (0x1000, 0x2000)),
        (0, (0x1000, 0x2000)),
    ] {
        check_state(
            &mut t,
            &format!("count {count}, car {car:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &[0x5A; 16]);
                game_write(CHECK_COUNT_RVA, count);
                game_write(CAR_X_RVA, car.0);
                game_write(CAR_Z_RVA, car.1);
            },
            &regions,
            &rust,
        );
    }
    drop(line);
    restore(&kept, &saved);
    t.finish()
}
