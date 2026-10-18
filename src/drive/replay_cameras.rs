//! `drive::replay_cameras` (0x460949): the two cars' replay cameras from the frame's head
//! (the first car block, `g_carBlockStore`: its first two dwords into `dword_6EF7D0`,
//! `6EF7D4`). For each car c (block c, its byte c of the head): the wish
//! `dword_719F14[byte]`; 0x23 .. 0x25 at least three frames after the last
//! (`dword_719FD0[c]`, frame `dword_5C2530`): a mode drawn (`drive::pick_replay_camera`)
//! into +0x684, +0x694 0, +0x690 the block's +8; another wish not already +0x684: it into
//! +0x688 and +0x684, the same two, the last frame 0. Then the option bits (+0x220) of
//! both from `dword_63DB7C`, `63DB80`, and the head's bytes 2 .. 5 into `dword_63DB74`,
//! `63DB78`, `63DB84`, `63DB88`.
//!
//! In:  nothing
//! Out: eax the head's byte 5, esi the first block, ebx and edx as the draws leave them;
//!      the rest kept; no flags read
//!
//! Callers: `drive::play_replay_frame`, `sub_46062E`.
//! Refuses what `pick_replay_camera` refuses (everything written put back).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): wishes, frames and last frames by lot, the random state.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, guard, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::replay_cameras", rva(0x460949), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const B: usize = 0x70F3EC; // g_carBlockStore
const NEXT: usize = 0xFE0;

fn byte(x: usize) -> u32 {
    unsafe { (a(x) as *const u8).read() as u32 }
}

fn steps(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    p(0x6EF7D0, g(B));
    p(0x6EF7D4, g(B + 4));
    for c in 0..2usize {
        let block = B + NEXT * c;
        let wish = g(0x719F14 + 4 * byte(B + c) as usize);
        r.eax = wish;
        if matches!(wish, 0x23..=0x25) {
            r.ebx = g(0x5C2530).wrapping_sub(g(0x719FD0 + 4 * c));
            if r.ebx >= 3 {
                p(0x719FD0 + 4 * c, g(0x5C2530));
                unsafe { crate::drive::pick_replay_camera::pick_replay_camera(&r) }?.apply(&mut r);
                p(block + 0x684, r.edx);
                p(block + 0x694, 0);
                r.eax = g(block + 8);
                p(block + 0x690, r.eax);
            }
        } else if wish != g(block + 0x684) {
            p(block + 0x688, wish);
            p(block + 0x684, wish);
            p(block + 0x694, 0);
            r.eax = g(block + 8);
            p(block + 0x690, r.eax);
            p(0x719FD0 + 4 * c, 0);
        }
    }
    p(B + 0x220, g(0x63DB7C));
    p(B + NEXT + 0x220, g(0x63DB80));
    for (k, to) in [0x63DB74usize, 0x63DB78, 0x63DB84, 0x63DB88]
        .iter()
        .enumerate()
    {
        r.eax = byte(B + 2 + k);
        p(*to, r.eax);
    }
    r.esi = a(B);
    Ok(Left::nothing()
        .with(EAX, r.eax)
        .with(crate::framework::EBX, r.ebx)
        .with(crate::framework::EDX, r.edx)
        .with(ESI, r.esi))
}

/// What it writes.
pub fn regions() -> Vec<Region> {
    vec![
        (a(0x6EF7D0), 8),
        (a(0x719FD0), 8),
        (a(B + 0x220), 4),
        (a(B + 0x684), 0x14),
        (a(B + NEXT + 0x220), 4),
        (a(B + NEXT + 0x684), 0x14),
        (a(0x63DB74), 0x18),
        (
            crate::framework::game_addr(crate::math::next_random::STATE_RVA) as u32,
            4,
        ),
    ]
}

/// The Rust version.
pub unsafe fn replay_cameras(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || steps(entry))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { replay_cameras(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend([
        (a(B), 8),
        (a(B + 8), 4),
        (a(B + NEXT + 8), 4),
        (a(0x719F14), 0x400),
        (a(0x5C2530), 4),
        (a(0x63DB7C), 8),
        (a(0x63DB94), 4),
    ]);
    let saved = capture(&all);
    for case in 0..16u32 {
        let set = || {
            p(B, 0x0504_0100 | ((case % 4) << 8) | (case % 3));
            p(B + 4, 0x0706);
            for k in 0..4usize {
                p(
                    0x719F14 + 4 * k,
                    [0x23u32, 0x24, 7, 0x25][(k + case as usize) % 4],
                );
            }
            p(0x5C2530, 10 + case);
            p(0x719FD0, [0u32, 9, 12][(case % 3) as usize]);
            p(0x719FD4, [11u32, 0][(case % 2) as usize]);
            p(B + 0x684, [7u32, 3][(case % 2) as usize]);
            p(B + NEXT + 0x684, 7);
            p(B + 8, 0x1234);
            p(B + NEXT + 8, 0x5678);
            unsafe {
                crate::framework::game_write(
                    crate::math::next_random::STATE_RVA,
                    case.wrapping_mul(0x2545_F491) | 1,
                )
            };
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
