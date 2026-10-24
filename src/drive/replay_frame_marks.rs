//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `dword_5C2530`'s two marks
//! (`unk_963BB4[frame]`, 8 bytes) into `dword_9613A4` .. (in `pusha` / `popa`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `sub_41D5FB`, `sub_41D846`, `sub_46020A`.
//! Self-test: the original and the Rust version on frames and marks by lot (saved and put
//! back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::flow::{a, g, p};

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks() {
    let at = a(0x963BB4).wrapping_add(rc2k_core::drive::replay_frames::marks_offset(g(0x5C2530)));
    for k in 0..2u32 {
        p(0x9613A4 + 4 * k as usize, unsafe {
            ((at + 4 * k) as *const u32).read_unaligned()
        });
    }
}
