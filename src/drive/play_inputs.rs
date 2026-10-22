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

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::play_inputs", rva(0x460162), thunk, &ORIGINAL).self_test(self_test);

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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { play_inputs(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(a(WORDS), 16)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![
        (a(WORDS), 16),
        (a(CURRENT), 4),
        (a(0x6EF7E4), 4),
        (a(TAPE), 0x80),
    ];
    let saved = capture(&all);
    for case in 0..12u32 {
        let set = || {
            p(CURRENT, case % 3);
            p(0x6EF7E4, case % 4);
            for k in 0..0x20usize {
                p(TAPE + 4 * k, (k as u32 + case).wrapping_mul(0x9E37_79B9));
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
