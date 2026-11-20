//! What the tests share to put the game's state back, by store name and field path rather
//! than by address: `state` (a snapshot of every store).

pub(crate) mod frozen;
pub(crate) mod own;
pub(crate) mod paths;
mod race_places;
pub(crate) mod race_vectors;
mod registry;
pub(crate) mod state;
pub(crate) mod vectors;

/// The lock of the tests that write the process-wide statics (`logfile`, `files`, `memory`,
/// `fatal`, `workers`' parts and the like: the stores the registry lists whose names do not begin
/// with `owners.`) or that read their first values: tests run in parallel threads.
///
/// A test whose state is only the test thread's own owners (`own::owners()`, `Owners::the()`, the
/// registry's `owners.*` stores, the race block's places the vectors name, the control word of the
/// x87) takes no lock: each thread has its own copy, and a lock there made every vector test wait
/// for the others. Hold the lock only around what touches a process-wide static, and never hold it
/// while waiting for another thread.
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// FNV-1a over `bytes`, 64 bits: the plain hasher the tests' expected values use.
pub fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3)
    })
}
