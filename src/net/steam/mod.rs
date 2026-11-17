//! Network play through Steam: sessions are Steam lobbies, and a joined machine's connection to
//! the host runs over Steam's relay (found by Steam ID), carrying the same protocol as TCP.
//!
//! The game's own application number is not Steam's: until the game has one of its own it runs
//! as Spacewar (480), which every Steam account has, so friends see "Spacewar" playing.
//!
//! Built only with the `steam` feature and not for 64-bit ARM Linux (the handheld has no Steam);
//! otherwise `absent` answers that Steam is not there and nothing else changes.

/// The Steam entry's provider GUID in the connection list (ours: DirectPlay has none for Steam).
pub const PROVIDER: [u8; 16] = *b"RC2K-STEAM-PROV\0";
/// The entry's name in the connection list.
pub const CONNECTION_NAME: &[u8] = b"Steam (friends and lobbies)\0";

#[cfg(any(
    test,
    all(
        feature = "steam",
        not(all(target_os = "linux", target_arch = "aarch64"))
    )
))]
mod lobby_data;
#[cfg(all(
    feature = "steam",
    not(all(target_os = "linux", target_arch = "aarch64"))
))]
pub use lobby_data::*;

#[cfg(all(
    feature = "steam",
    not(all(target_os = "linux", target_arch = "aarch64"))
))]
mod real;
#[cfg(all(
    feature = "steam",
    not(all(target_os = "linux", target_arch = "aarch64"))
))]
pub use real::*;

#[cfg(not(all(
    feature = "steam",
    not(all(target_os = "linux", target_arch = "aarch64"))
)))]
mod absent;
#[cfg(not(all(
    feature = "steam",
    not(all(target_os = "linux", target_arch = "aarch64"))
)))]
pub use absent::*;
