//! Network play through Steam: sessions are Steam lobbies, and a joined machine's connection to
//! the host runs over Steam's relay (found by Steam ID), carrying the same protocol as TCP.
//!
//! The game's own application number is not Steam's: until the game has one of its own it runs
//! as Spacewar (480), which every Steam account has, so friends see "Spacewar" playing.
//!
//! Valve's library is not linked: `flat` loads it while the program runs, from the program's
//! folder or the system's search path, so one build serves with and without Steam; when it is
//! not there (or no Steam client runs) Steam is not available, it is not listed, and nothing
//! else changes. Built for Windows, Linux and macOS; not for 64-bit ARM Linux (the handheld has
//! no Steam) nor the web, where `absent` answers that Steam is not there. (`steam_support` is
//! the build script's name for the targets that have it.)

/// The Steam entry's provider GUID in the connection list (ours: DirectPlay has none for Steam).
pub const PROVIDER: [u8; 16] = *b"RC2K-STEAM-PROV\0";
/// The entry's name in the connection list.
pub const CONNECTION_NAME: &[u8] = b"Steam (friends and lobbies)\0";

// Without Steam's library (aarch64) only the tests reach the callbacks' readers.
#[cfg(any(test, steam_support))]
#[cfg_attr(not(steam_support), allow(dead_code))]
mod callbacks;
#[cfg(any(test, steam_support))]
mod lobby_data;
#[cfg(steam_support)]
pub use lobby_data::*;

#[cfg(steam_support)]
mod flat;
#[cfg(steam_support)]
mod real;
#[cfg(steam_support)]
pub use real::*;

#[cfg(not(steam_support))]
mod absent;
#[cfg(not(steam_support))]
pub use absent::*;
