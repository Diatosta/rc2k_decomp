//! The audio thread's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`):
//! IDA 0x4F8294 .. 0x506E6C, the sound mode and volumes, the voices (0x200 of 0x70 bytes at
//! 0x4F834C), the mixer's channels and cursors, the sample file and buffer, the streams,
//! the command counters, DirectSound / A3D / EAX's objects and settings, the quality modes
//! and the audio options the advanced page edits; and the sample records
//! (0x50A7A4 .. 0x511AE4, 0x1CD of 0x40 bytes); and the queue of commands for the thread
//! (0x9566F8 .. 0x95E6F8, 0x400 records of 0x20 bytes, bss).
//!
//! Each region is one block with the image's layout (`crate::store`): gaps and unreferenced
//! bytes kept, first values the image's (`store_data.rs`, generated). The voices' pool and
//! the records start as the image holds them.
//!
//! Threads: the audio thread and the game's thread share these as the image's bytes were
//! shared (volatile reads and writes; the command count is added to atomically,
//! `sound::queue`).

use crate::block;

block!(pub mod state {
    start: 0x4F8294,
    end: 0x506E6C,
    runs: crate::audio::store_data::STATE_RUNS,
});

block!(pub mod samples {
    start: 0x50A7A4,
    end: 0x511AE4,
    runs: crate::audio::store_data::SAMPLE_RUNS,
});

block!(pub mod queue {
    start: 0x9566F8,
    end: 0x95E6F8,
    runs: &[],
});

pub use state::Slot;
pub type SampleSlot = samples::Slot;
pub type QueueSlot = queue::Slot;

/// `g_eaxEnvironment` (IDA 0x6D1DD4): the EAX environment the race asks for (the audio
/// thread reads it).
pub static ENVIRONMENT: crate::session::Dword = crate::session::Dword::new(0);

#[cfg(test)]
mod tests {
    #[test]
    fn the_environment_starts_as_the_games() {
        assert_eq!(
            super::ENVIRONMENT.get().to_le_bytes().to_vec(),
            crate::moved::image_bytes(0x6D1DD4, 4)
        );
    }
}
