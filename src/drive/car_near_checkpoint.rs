//! `drive::car_near_checkpoint` (0x4375F2): whether a car (esi) in the stage (+0x25C not
//! 0) is within 3 checkpoints of eax: CF when |+0x98 - eax| (`neg` of a negative) less 4
//! has its sign set.
//!
//! In:  esi a car block, eax a checkpoint
//! Out: CF; ebx the distance when +0x25C is not 0; the rest kept (ZF not read: `jb`)
//!
//! Caller: `drive::checkpoint_taken` (four cars). Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original on a test car, 40 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use rc2k_core::physics::line_places::car_near;

/// The Rust core: whether the car block at `car` is near `checkpoint` (eax; the CF).
/// The distance it leaves in ebx is read by no caller (`checkpoint_taken` keeps ebx).
// Converted: `rc2k_core::physics::line_places::car_near`.
pub unsafe fn near_at(car: u32, checkpoint: u32) -> bool {
    let get = |o: u32| unsafe { ((car + o) as *const u32).read_unaligned() };
    car_near(get(0x25C) != 0, || get(0x98), checkpoint).is_some_and(|n| n.1)
}
