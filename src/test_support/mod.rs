//! What the tests share to put the game's state back, by store name and field path rather
//! than by address: `state` (a snapshot of every store).

pub mod frozen;
pub mod paths;
pub mod race_places;
pub mod race_vectors;
mod registry;
pub mod state;
pub mod vectors;

/// The lock every test holds that writes the game's statics (and restores them), or reads
/// their first values: tests run in parallel threads.
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
