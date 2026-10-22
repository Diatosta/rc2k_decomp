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
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::rva;

/// dword_4E0140: the generator's state.
pub const STATE_RVA: usize = rva(0x4E0140);

// ---- Rust version ----

/// The bridge (`src/bridge.rs`): the generator loaded from `dword_4E0140`, `f` run on it,
/// its state stored back.
pub fn with_random<R>(f: impl FnOnce(&mut rc2k_core::math::random::Random) -> R) -> R {
    static STATE: crate::bridge::Global<u32> = crate::bridge::Global::at(0x4E_0140);
    let mut random = rc2k_core::math::random::Random { state: STATE.get() };
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
