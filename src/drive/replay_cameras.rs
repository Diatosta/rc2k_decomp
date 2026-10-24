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
//! Callers: `drive::play_replay_frame`, `sub_46062E`. Refuses what `pick_replay_camera`
//! refuses (everything written put back).

use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
use rc2k_core::drive::replay_views::{WishStep, wish_step};

const B: usize = 0x70F3EC; // g_carBlockStore
const NEXT: usize = 0xFE0;

fn byte(x: usize) -> u32 {
    unsafe { (a(x) as *const u8).read() as u32 }
}

// Converted: `rc2k_core::drive::replay_views::wish_step`.
fn steps() -> Result<(), Refused> {
    p(0x6EF7D0, g(B));
    p(0x6EF7D4, g(B + 4));
    for c in 0..2usize {
        let block = B + NEXT * c;
        let wish = g(0x719F14 + 4 * byte(B + c) as usize);
        match wish_step(wish, g(0x5C2530), g(0x719FD0 + 4 * c), || g(block + 0x684)) {
            WishStep::Draw(_) => {
                p(0x719FD0 + 4 * c, g(0x5C2530));
                // `Draw` comes only for the wishes that draw: the mode is there.
                if let Some(mode) =
                    unsafe { crate::drive::pick_replay_camera::pick_replay_camera(wish) }?
                {
                    p(block + 0x684, mode);
                }
                p(block + 0x694, 0);
                p(block + 0x690, g(block + 8));
            }
            WishStep::Nothing(_) => {}
            WishStep::Set => {
                p(block + 0x688, wish);
                p(block + 0x684, wish);
                p(block + 0x694, 0);
                p(block + 0x690, g(block + 8));
                p(0x719FD0 + 4 * c, 0);
            }
        }
    }
    p(B + 0x220, g(0x63DB7C));
    p(B + NEXT + 0x220, g(0x63DB80));
    for (k, to) in [0x63DB74usize, 0x63DB78, 0x63DB84, 0x63DB88]
        .iter()
        .enumerate()
    {
        p(*to, byte(B + 2 + k));
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn replay_cameras() -> Result<(), Refused> {
    steps()
}
