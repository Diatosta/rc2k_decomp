//! `math::next_random` (0x4018B4): the game's random numbers.
//!
//! In:  nothing
//! Out: eax = the next number; everything else preserved
//! One step of the generator on its state `dword_4E0140` (`math::random_step`, 0x4018C8,
//! with 0 mixed in): the low byte is changed by three bits of the number (bit 22, then
//! bit 2 and bit 1 of the byte as it then is, each flipping bit 0), the whole is turned
//! right by one, and what is mixed in is `xor`ed on. 23 callers, and
//! `math::random_below` (73 more).
//! Shadow mode: the Rust step, the state put back, the original's, compared. Another
//! thread drawing a number in between would show as a mismatch that is none.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::next_random", rva(0x4018B4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// dword_4E0140: the generator's state.
pub const STATE_RVA: usize = rva(0x4E0140);

// ---- Rust version ----

/// `sub_4018C8`.
pub fn random_step(state: u32, mixed_in: u32) -> u32 {
    let mut low = state as u8;
    low ^= (state & 0x40_0000 != 0) as u8;
    low ^= (low & 4 != 0) as u8;
    low ^= (low & 2 != 0) as u8;
    (state & !0xFF | low as u32).rotate_right(1) ^ mixed_in
}

pub unsafe fn next_random() -> u32 {
    unsafe {
        let next = random_step(game_read(STATE_RVA), 0);
        game_write(STATE_RVA, next);
        next
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { next_random() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
}

// ---- self-test ----

pub const TEST_STATES: [u32; 12] = [
    0,
    1,
    2,
    4,
    6,
    7,
    0x40_0000,
    0x40_0006,
    0xFFFF_FFFF,
    0x8000_0001,
    0x1234_5678,
    0xDEAD_BEEF,
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(STATE_RVA)];
    let saved = capture(&regions);
    for state in TEST_STATES {
        check_state(
            &mut t,
            &format!("state={state:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(STATE_RVA, state) },
            &regions,
            &rust,
        );
    }
    // A run of numbers from one state.
    let mut state = 0x2026_1012u32;
    for _ in 0..200 {
        let now = state;
        check_state(
            &mut t,
            &format!("state={now:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(STATE_RVA, now) },
            &regions,
            &rust,
        );
        state = random_step(state, 0);
    }
    restore(&regions, &saved);
    t.finish()
}
