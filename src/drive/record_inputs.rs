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

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::record_inputs", rva(0x4601A8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);
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

/// The tape's frame written.
pub fn regions() -> Vec<Region> {
    vec![
        (a(WORDS), 16),
        (a(TAPE).wrapping_add(g(0x6EF7DC).wrapping_mul(16)), 16),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![
        (a(WORDS), 16),
        (a(CURRENT), 4),
        (a(0x6EF7DC), 4),
        (a(TAPE), 0x80),
    ];
    let saved = capture(&all);
    for case in 0..12u32 {
        let set = || {
            p(CURRENT, case % 3);
            p(0x6EF7DC, case % 4);
            for k in 0..4usize {
                p(
                    WORDS + 4 * k,
                    (k as u32 + case).wrapping_mul(0x9E37_79B9) >> (case % 5 * 5),
                );
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
