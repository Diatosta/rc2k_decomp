//! `drive::pick_replay_camera` (0x460AA6): for a replay's camera wish eax (0x23, 0x24,
//! 0x25: kinds 0, 1, 2), a camera mode drawn (`math::next_random` x 0x34, high dword) until
//! its record (`off_606320[mode]`) is of that kind (0x23: until it is NOT of kind 0, the
//! original's `jz`; notes/bugs.md); modes 2 and 3 become 4 when a car's
//! kind (+0x24, `g_carBlockStore`) is past 0x16 (the second car's only with two cars,
//! `dword_63DB94` not 1). Other wishes: nothing.
//!
//! In:  eax the wish
//! Out: edx the mode; eax the last draw's product, ebx its record; the rest kept; no flags
//!      read
//!
//! Caller: `drive::replay_cameras`.
//! Refuses (before anything) a wish no mode stops (the original draws for ever).
//! Self-test: the original and the Rust version on the game's records, the random state
//! saved and put back, the wishes and car kinds.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use rc2k_core::drive::replay_views::{MODES as MODE_COUNT, draw, fits, kept_mode, wish_kind};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::pick_replay_camera", rva(0x460AA6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MODES: usize = 0x606320; // off_606320
const B: usize = 0x70F3EC; // g_carBlockStore

fn kind_of(mode: u32) -> u32 {
    unsafe { (g(MODES + 4 * mode as usize) as *const u32).read_unaligned() }
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_views::{wish_kind, fits, draw, kept_mode}`.
pub unsafe fn pick_replay_camera(entry: &Regs) -> Result<Left, Refused> {
    let Some(kind) = wish_kind(entry.eax) else {
        return Ok(Left::nothing());
    };
    let fits = |mode: u32| fits(kind, kind_of(mode));
    if !(0..MODE_COUNT).any(fits) {
        return Err(BadInput("no camera mode the wish stops at").into());
    }
    let (eax, edx) = loop {
        let (eax, edx) = draw(unsafe { crate::math::next_random::next_random() });
        if fits(edx) {
            break (eax, edx);
        }
    };
    let ebx = g(MODES + 4 * edx as usize);
    let edx = kept_mode(
        edx,
        || g(B + 0x24),
        || g(0x63DB94) != 1,
        || g(B + 0xFE0 + 0x24),
    );
    Ok(Left::nothing().with(EAX, eax).with(EBX, ebx).with(EDX, edx))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { pick_replay_camera(r) }
}

fn regions() -> Vec<Region> {
    vec![(crate::framework::game_addr(STATE_RVA) as u32, 4)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend([(a(B + 0x24), 4), (a(B + 0xFE0 + 0x24), 4), (a(0x63DB94), 4)]);
    let saved = capture(&all);
    for case in 0..12u32 {
        let wish = [0x23u32, 0x24, 0x25, 0x10][(case % 4) as usize];
        let kind = wish.wrapping_sub(0x23);
        if wish != 0x10 && !(0..0x34).any(|m| (kind_of(m) == kind) != (kind == 0)) {
            continue;
        }
        let set = || {
            unsafe { crate::framework::game_write(STATE_RVA, case.wrapping_mul(0x2545_F491) | 1) };
            p(B + 0x24, [3u32, 0x17][(case / 4 % 2) as usize]);
            p(B + 0xFE0 + 0x24, [0x20u32, 1][(case / 8) as usize]);
            p(0x63DB94, case % 2 + 1);
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                eax: wish,
                ..Regs::SENTINEL
            },
            &set,
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
