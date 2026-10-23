//! `drive::record_inputs` (0x4601A8): for cars 0 and 1 the input words onto the replay's
//! tape: ebx and the packed rest (`drive::pack_inputs`) into frame `dword_6EF7DC` (the
//! car's 8 of its 16 bytes), and the words given back as the tape keeps them
//! (`drive::unpack_inputs`).
//!
//! In:  nothing
//! Out: eax .. edx the words, esi the tape's offset (cars 0, 1); the rest kept; no flags
//!      read
//!
//! Caller: `drive::record_inputs_if`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): cars 0 .. 2, frames and words by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, ESI, Regs};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

const WORDS: usize = 0x6138C4; // dword_6138C4 .. 6138D0: the input words
const TAPE: usize = 0x7194BC; // dword_7194BC / 7194C0: the replay's inputs, 16 bytes a frame
const CURRENT: usize = 0x71B3F8; // g_currentCar

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub unsafe fn record_inputs() -> Left {
    let car = g(CURRENT);
    if !on_tape(car) {
        return Left::nothing();
    }
    let (eax, ebx, ecx, edx) = (g(WORDS), g(WORDS + 4), g(WORDS + 8), g(WORDS + 12));
    let packed = crate::drive::pack_inputs::pack_inputs(eax, ecx, edx);
    let esi = tape_offset(g(0x6EF7DC), car);
    let at = |o: u32| a(TAPE).wrapping_add(esi).wrapping_add(o) as *mut u32;
    unsafe {
        at(0).write_unaligned(ebx);
        at(4).write_unaligned(packed);
    }
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(packed);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        p(WORDS + 4 * k, *v);
    }
    Left::nothing()
        .with(EAX, eax)
        .with(EBX, ebx)
        .with(ECX, ecx)
        .with(EDX, edx)
        .with(ESI, esi)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { record_inputs() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
