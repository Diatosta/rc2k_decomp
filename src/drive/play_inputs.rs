//! `drive::play_inputs` (0x460162): for cars 0 and 1 the input words from the replay's
//! tape: frame `dword_6EF7E4` (16 bytes; the car's 8) has ebx and the packed word, taken
//! apart (`drive::unpack_inputs`) into `dword_6138C4` .. (eax, ebx, ecx, edx).
//!
//! In:  nothing
//! Out: eax .. edx the words, esi the tape's offset (cars 0, 1); the rest kept; no flags
//!      read
//!
//! Callers: `drive::play_inputs_if`, `sub_460835`.

use crate::physics::flow::{a, g, p};
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

const WORDS: usize = 0x6138C4; // dword_6138C4 .. 6138D0: the input words
const TAPE: usize = 0x7194BC; // dword_7194BC / 7194C0: the replay's inputs, 16 bytes a frame
const CURRENT: usize = 0x71B3F8; // g_currentCar

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub unsafe fn play_inputs() {
    let car = g(CURRENT);
    if !on_tape(car) {
        return;
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
}
