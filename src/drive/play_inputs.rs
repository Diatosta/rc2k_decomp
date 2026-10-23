//! `drive::play_inputs` (0x460162): for cars 0 and 1 the input words from the replay's
//! tape: frame `dword_6EF7E4` (16 bytes; the car's 8) has ebx and the packed word, taken
//! apart (`drive::unpack_inputs`) into `dword_6138C4` .. (eax, ebx, ecx, edx).
//!
//! In:  nothing
//! Out: eax .. edx the words, esi the tape's offset (cars 0, 1); the rest kept; no flags
//!      read
//!
//! Callers: `drive::play_inputs_if`, `sub_460835`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): cars 0 .. 2, frames and words by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, EBX, ECX, EDX, ESI, Regs};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

const WORDS: usize = 0x6138C4; // dword_6138C4 .. 6138D0: the input words
const TAPE: usize = 0x7194BC; // dword_7194BC / 7194C0: the replay's inputs, 16 bytes a frame
const CURRENT: usize = 0x71B3F8; // g_currentCar

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub unsafe fn play_inputs(entry: &Regs) -> Left {
    let car = g(CURRENT);
    if !on_tape(car) {
        return Left::nothing();
    }
    let esi = tape_offset(g(0x6EF7E4), car);
    let at = |o: u32| a(TAPE).wrapping_add(esi).wrapping_add(o);
    let ebx = unsafe { (at(0) as *const u32).read_unaligned() };
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(unsafe {
        (at(4) as *const u32).read_unaligned()
    });
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        p(WORDS + 4 * k, *v);
    }
    let _ = entry;
    Left::nothing()
        .with(EAX, eax)
        .with(EBX, ebx)
        .with(ECX, ecx)
        .with(EDX, edx)
        .with(ESI, esi)
}
