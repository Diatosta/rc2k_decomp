//! `math::next_random` (0x4018B4): the game's random numbers.
//!
//! In:  nothing
//! Out: eax = the next number, ZF set when it is 0 (read by `race::nonzero_random`);
//!      everything else preserved
//! One step of the generator on its state `dword_4E0140` (`math::random_step`, 0x4018C8,
//! with 0 mixed in): the low byte is changed by three bits of the number (bit 22, then
//! bit 2 and bit 1 of the byte as it then is, each flipping bit 0), the whole is turned
//! right by one, and what is mixed in is `xor`ed on. 23 callers, and
//! `math::random_below` (73 more).
//! Shadow mode: the Rust step, the state put back, the original's, compared. Another
//! thread drawing a number in between would show as a mismatch that is none.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
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

/// `sub_4018C8` (converted: `rc2k_core::random::step`).
pub fn random_step(state: u32, mixed_in: u32) -> u32 {
    rc2k_core::random::step(state, mixed_in)
}

/// The bridge (`src/bridge.rs`): the generator loaded from `dword_4E0140`, `f` run on it,
/// its state stored back.
pub fn with_random<R>(f: impl FnOnce(&mut rc2k_core::random::Random) -> R) -> R {
    static STATE: crate::bridge::Global<u32> = crate::bridge::Global::at(0x4E_0140);
    let mut random = rc2k_core::random::Random { state: STATE.get() };
    let value = f(&mut random);
    STATE.set(random.state);
    value
}

/// The thread that owns the generator during a self-test that draws (0: none). Another
/// thread asking for a number meanwhile waits: a thread let through the pause (the log
/// writer, a pause timeout) would put the test's two sides on different numbers.
static OWNER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn this_thread() -> u32 {
    unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() }
}

/// Runs `f` (a self-test) with the generator to this thread alone.
pub fn own_random<R>(f: impl FnOnce() -> R) -> R {
    use std::sync::atomic::Ordering;
    /// Gives the generator back however `f` ends (a panic too), or the game would wait.
    struct Release;
    impl Drop for Release {
        fn drop(&mut self) {
            OWNER.store(0, Ordering::SeqCst);
        }
    }
    OWNER.store(this_thread(), Ordering::SeqCst);
    let _release = Release;
    f()
}

/// Waits while another thread owns the generator (`own_random`).
pub fn wait_for_random() {
    use std::sync::atomic::Ordering;
    loop {
        let owner = OWNER.load(Ordering::SeqCst);
        if owner == 0 || owner == this_thread() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// The numbers drawn so far (for tests that compare how many each side drew).
pub static DRAWS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub unsafe fn next_random() -> u32 {
    wait_for_random();
    DRAWS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    with_random(|r| r.draw())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { next_random() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    wait_for_random();
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
    // `random_step` ends in `xor eax, ebx` (ebx 0): its caller `race::nonzero_random`
    // (0x444C14) reads the zero flag (`jnz`).
    r.set_zero(r.eax == 0);
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
