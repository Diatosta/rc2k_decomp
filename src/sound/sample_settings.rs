//! `sound::sample_settings` (0x40BA71): a sound's volume and rate, the sample's own where
//! asked for and held to what the mixer takes.
//!
//! In:  eax the sample (a record of `g_sampleRecords`, 0x40 bytes each), ecx the volume,
//!      edx the rate; -0x6F (`0xFFFFFF91`) in either: the sample's own, -0x70 for the
//!      rate: its buffer's
//! Out: ecx the volume, edx the rate; everything else preserved
//! A sample without a buffer (+8 null): ecx 0, edx 44100. Else ecx -0x6F takes the
//! record's +0x28, then is held to 0 .. 0x40 (signed); edx -0x6F takes the record's
//! +0x30, then (so a record's -0x70 chains) -0x70 the buffer's +0x24, then is held to
//! 100 .. 200000 (signed).
//! Five callers (the race sounds' turn). Refuses a sample past the 0x1CD records (the
//! original reads past them; not worked out).
//! Shadow mode: both, the registers compared (`dispatch_state`, nothing written).
//! Self-test: every record with a set of volumes and rates, and a record of the test's
//! (record 0, put back) for the branches the game's may not reach.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{ECX, EDX, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

pub const RECORDS_RVA: usize = rva(0x50A7A4); // g_sampleRecords
pub const RECORDS: u32 = 0x1CD;
pub const RECORD_LEN: usize = 0x40;
/// "The sample's own".
pub const OWN: u32 = 0xFFFF_FF91;
/// "The buffer's" (rates only).
pub const BUFFER_RATE: u32 = 0xFFFF_FF90;

/// The record of sample `n`, or a refusal past the records.
pub fn record(n: u32) -> Result<*const u32, BadInput> {
    if n >= RECORDS {
        return Err(BadInput("a sample past the 0x1CD records"));
    }
    Ok((game_addr(RECORDS_RVA) + n as usize * RECORD_LEN) as *const u32)
}

/// The dword at byte `at` of `p`.
pub fn word(p: *const u32, at: usize) -> u32 {
    unsafe { (p as *const u8).add(at).cast::<u32>().read_unaligned() }
}

/// The rate as both leaves take it: -0x6F the record's +0x30, then -0x70 the buffer's
/// +0x24; held to 100 .. 200000.
/// The record's and its first voice's figures (`None`: no voice, not loaded).
pub fn loaded(record: *const u32) -> Option<rc2k_core::audio::samples::Loaded> {
    let voice = word(record, 8) as *const u32;
    if voice.is_null() {
        return None;
    }
    Some(rc2k_core::audio::samples::Loaded {
        voice_kind: word(record, 0x14),
        volume: word(record, 0x28),
        frequency: word(record, 0x30),
        own_frequency: word(voice, 0x24),
        format: word(voice, 0x28),
        frames: word(voice, 0x2C),
    })
}

// ---- Rust version ----

/// (volume, rate).
pub fn sample_settings(sample: u32, volume: u32, rate: u32) -> Result<(u32, u32), BadInput> {
    // Converted: `rc2k_core::audio::samples::settings`.
    let record = record(sample)?;
    Ok(rc2k_core::audio::samples::settings(
        volume,
        rate,
        loaded(record).as_ref(),
    ))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (volume, rate) = sample_settings(r.eax, r.ecx, r.edx)?;
    Ok(Left::nothing().with(ECX, volume).with(EDX, rate))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

pub const TEST_VOLUMES: [u32; 6] = [OWN, BUFFER_RATE, 0, 0x20, 0x41, 0xFFFF_FFF0];
pub const TEST_RATES: [u32; 8] = [OWN, BUFFER_RATE, 0, 99, 22050, 200_001, 0x8000_0000, 7];
