//! `drive::keep_replay_frame` (0x45FFB6): the replay's frame and tenth (`dword_6EF7D8`,
//! `dword_6EF7DC`, 0 when playing, `dword_5C5CE0` 2; else the first block's +8 / 100 and
//! its remainder); at a whole frame the four car blocks (0x3F80 bytes from
//! `g_carBlockStore`) kept in slot frame & 7 of `unk_6EF7E8`, and while recording
//! (`dword_5C5CE0` 1) `drive::save_replay_cars`.
//!
//! In:  nothing
//! Out: eax / edx / ebx as the `div` leaves them (not playing), at a whole frame eax the
//!      slot's offset (`imul`), ecx 0, esi / edi past the copy, or as `save_replay_cars` leaves them; the rest kept; no
//!      flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): modes, whole frames and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI, Regs};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::replay_frames::{
    PLAYING, RECORDING, SLOT_SIZE, TENTHS, frame_and_tenth, slot_offset,
};

const B: usize = 0x70F3EC; // g_carBlockStore
const SLOTS: usize = 0x6EF7E8; // unk_6EF7E8: eight of 0x3F80
const SIZE: usize = SLOT_SIZE as usize;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{frame_and_tenth, slot_offset}`.
pub unsafe fn keep_replay_frame() -> Left {
    let mut left = Left::nothing();
    p(0x6EF7D8, 0);
    p(0x6EF7DC, 0);
    if g(0x5C5CE0) != PLAYING {
        let (frame, tenth) = frame_and_tenth(g(B + 8));
        p(0x6EF7D8, frame);
        p(0x6EF7DC, tenth);
        left = left.with(EAX, frame).with(EDX, tenth).with(EBX, TENTHS);
    }
    if g(0x6EF7DC) == 0 {
        let offset = slot_offset(g(0x6EF7D8));
        let to = a(SLOTS) + offset;
        unsafe { std::ptr::copy(a(B) as *const u8, to as *mut u8, SIZE) };
        left = left
            .with(EAX, offset)
            .with(ESI, a(B) + SIZE as u32)
            .with(EDI, to + SIZE as u32)
            .with(ECX, 0);
        if g(0x5C5CE0) == RECORDING {
            let saved = crate::drive::save_replay_cars::save_replay_cars();
            for &(reg, v) in &saved.regs {
                left = left.with(reg, v);
            }
        }
    }
    left
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { keep_replay_frame() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
