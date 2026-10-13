//! `scene::next_noise` (0x440F3E): the next number of the scene's own generator.
//!
//! In:  nothing
//! Out: eax = the number; everything else preserved
//!
//! One step on the state `dword_614C54` (`scene::noise_step`, 0x440F4E, inlined): bit
//! 22, then bits 2 and 1 of the low byte as it then is, each flip bit 0; the whole
//! turned right by one; then `xor` 0x87654321. `math::next_random`'s step with that
//! last `xor` added.
//! Five callers (two in the engine's tables, three unread).
//! Self-test: 200 states.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::next_noise", rva(0x440F3E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const STATE_RVA: usize = rva(0x614C54); // dword_614C54

// ---- Rust version ----

/// `scene::noise_step` (0x440F4E).
pub fn noise_step(mut state: u32) -> u32 {
    state ^= (state >> 22) & 1;
    state ^= (state >> 2) & 1;
    state ^= (state >> 1) & 1;
    state.rotate_right(1) ^ 0x8765_4321
}

pub unsafe fn next_noise() -> u32 {
    unsafe {
        let state = noise_step(game_read(STATE_RVA));
        game_write(STATE_RVA, state);
        state
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { next_noise() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [at(STATE_RVA)];
    let saved = capture(&regions);
    let mut state = 0x0012_D687u32;
    for case in 0..200u32 {
        let from = match case {
            0 => 0,
            1 => 0xFFFF_FFFF,
            2 => 0x0040_0000,
            3 => 6,
            4 => 0x0040_0006,
            _ => {
                state = noise_step(state).wrapping_mul(0x9E37_79B9);
                state
            }
        };
        check_state(
            &mut t,
            &format!("state {from:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(STATE_RVA, from) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
