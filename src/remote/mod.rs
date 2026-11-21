//! The test driver's door. The game calls these functions without any `cfg`: with the cargo
//! feature `remote` they reach the driver (`driver.rs`: commands from `RC2K_REMOTE`, held races,
//! hashes of what a stage or a mix leaves, `RC2K_*` switches), and without it (the shipped build)
//! they are constants and no-ops (`off.rs`), so the shipped code is the code the tests run, less
//! the driver's own lines.

#[cfg(feature = "remote")]
mod driver;
#[cfg(not(feature = "remote"))]
mod off;

#[cfg(feature = "remote")]
pub use driver::*;
#[cfg(not(feature = "remote"))]
pub use off::*;

/// Whether this build has the driver.
pub const BUILT: bool = cfg!(feature = "remote");
