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

use crate::physics::flow::g;
use crate::physics::tables_store::PHYSICS;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub unsafe fn record_inputs() {
    let car = RACE_WORK.current_car.get();
    if !on_tape(car) {
        return;
    }
    let (eax, ebx, ecx, edx) = (
        PHYSICS.input_words[0].get(),
        PHYSICS.input_words[1].get(),
        PHYSICS.input_words[2].get(),
        PHYSICS.input_words[3].get(),
    );
    let packed = crate::drive::pack_inputs::pack_inputs(eax, ecx, edx);
    let esi = tape_offset(g(0x6EF7DC), car);
    let at = |o: u32| {
        (REPLAY_WORK.replay_tape[0].addr() as u32)
            .wrapping_add(esi)
            .wrapping_add(o) as *mut u32
    };
    unsafe {
        at(0).write_unaligned(ebx);
        at(4).write_unaligned(packed);
    }
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(packed);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        PHYSICS.input_words[k].set(*v);
    }
}
