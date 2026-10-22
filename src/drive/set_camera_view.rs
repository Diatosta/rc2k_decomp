//! `drive::set_camera_view` (0x43A9BA): a block's (esi) view taken: +0x684 = the wanted
//! +0x688, +0x694, +0x69C, +0x698 0, +0x690 = the step count `dword_719FE0` (the work
//! copy's +8, whatever esi is). It starts with a `jmp` over a test that nothing reaches
//! (0x43A9BC: the same view kind by `dword_6063F0`, then +0x694 1 and +0x698 40).
//! By the look of it the camera views (`drive::next_camera_view` cycles +0x688; a
//! reading).
//!
//! In:  esi the block
//! Out: eax the step count; the rest kept; no flags read (callers go on with `mov`,
//!      `retn`)
//!
//! A `jmp` is its first instruction: `.private_copy` with `.manual_patch`.
//! Callers: `drive::next_camera_view`, `sub_43A649` (0x43A94F), and code at 0x43B62E.
//! Self-test: a copy of the original (its step count the test's) on a test block, 6
//! fillings.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, state_entry};
use rc2k_core::drive::camera_views::CameraView;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::set_camera_view", rva(0x43A9BA), thunk, &ORIGINAL)
    .private_copy(0x7E)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const STEP_RVA: usize = rva(0x719FE0); // dword_719FE0
pub const VIEW_LEN: u32 = 0x6A0;

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust version on the block at `s`, the step count at `step`.
// Converted: `rc2k_core::drive::camera_views::CameraView::take_wanted`.
pub unsafe fn set_view_at(s: u32, step: u32) -> Left {
    unsafe {
        let mut v = CameraView {
            wanted: (s.wrapping_add(0x688) as *const u32).read_unaligned(),
            ..CameraView::default()
        };
        let n = v.take_wanted((step as *const u32).read_unaligned());
        put(s + 0x684, v.view);
        put(s + 0x694, v.held);
        put(s + 0x69C, v.w69c);
        put(s + 0x698, v.w698);
        put(s + 0x690, v.taken_at);
        Left::nothing().with(EAX, n)
    }
}

/// The Rust version.
pub unsafe fn set_camera_view(s: u32) -> Left {
    unsafe { set_view_at(s, game_addr(STEP_RVA) as u32) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { set_camera_view(r.esi) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi + 0x684, 0x1C)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// A copy of the original whose step count is at `step`.
pub fn test_copy(step: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43A9BA,
            len: 0x7E,
            entry: &[0xEB, 0x1C, 0x8B, 0x9E, 0x84, 0x06, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x719FE0, step)],
        2,
    )
}

/// Fills a test block (`VIEW_LEN` bytes) by `seed`.
pub fn fill_block(s: u32, seed: u32) {
    let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
    for k in 0..VIEW_LEN / 4 {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
        unsafe { put(s + 4 * k, x) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; VIEW_LEN as usize / 4 + 1];
    let s = cells.as_mut_ptr() as u32;
    let step = s + VIEW_LEN;
    let copy = match test_copy(step) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(s, VIEW_LEN as usize + 4)];
    for seed in 1..=6u32 {
        let fill = || {
            fill_block(s, seed);
            unsafe { put(step, seed * 1000) };
        };
        fill();
        let before = Regs {
            esi: s,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill();
        let left = unsafe { set_view_at(s, step) };
        let mine = capture(&all);
        let regs_ok = left.matches(&before, &ro);
        t.check(regs_ok && theirs == mine, || {
            format!("seed {seed}: regs_ok={regs_ok}; the block differs")
        });
    }
    drop((copy, cells));
    t.finish()
}
