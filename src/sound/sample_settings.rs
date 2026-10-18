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

use crate::errors::{BadInput, Refused};
use crate::framework::{ECX, EDX, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::sample_settings", rva(0x40BA71), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
pub fn rate_of(record: *const u32, buffer: *const u32, rate: u32) -> u32 {
    let rate = if rate == OWN {
        word(record, 0x30)
    } else {
        rate
    };
    let rate = if rate == BUFFER_RATE {
        word(buffer, 0x24)
    } else {
        rate
    };
    (rate as i32).clamp(100, 200_000) as u32
}

// ---- Rust version ----

/// (volume, rate).
pub fn sample_settings(sample: u32, volume: u32, rate: u32) -> Result<(u32, u32), BadInput> {
    let record = record(sample)?;
    let buffer = word(record, 8) as *const u32;
    if buffer.is_null() {
        return Ok((0, 0xAC44));
    }
    let volume = if volume == OWN {
        word(record, 0x28)
    } else {
        volume
    };
    Ok((
        (volume as i32).clamp(0, 0x40) as u32,
        rate_of(record, buffer, rate),
    ))
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (volume, rate) = sample_settings(r.eax, r.ecx, r.edx)?;
    Ok(Left::nothing().with(ECX, volume).with(EDX, rate))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

pub const TEST_VOLUMES: [u32; 6] = [OWN, BUFFER_RATE, 0, 0x20, 0x41, 0xFFFF_FFF0];
pub const TEST_RATES: [u32; 8] = [OWN, BUFFER_RATE, 0, 99, 22050, 200_001, 0x8000_0000, 7];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let case = |t: &mut SelfTest, sample: u32, volume: u32, rate: u32, fill: &dyn Fn()| {
        check_state(
            t,
            &format!("sample {sample:#x}, volume {volume:#x}, rate {rate:#x}"),
            call_original,
            Regs {
                eax: sample,
                ecx: volume,
                edx: rate,
                ..Regs::SENTINEL
            },
            fill,
            &[],
            &rust,
        );
    };
    for sample in 0..RECORDS {
        for (k, &volume) in TEST_VOLUMES.iter().enumerate() {
            case(&mut t, sample, volume, TEST_RATES[k], &|| {});
        }
        case(&mut t, sample, OWN, TEST_RATES[7], &|| {});
        case(&mut t, sample, OWN, TEST_RATES[6], &|| {});
    }
    // Record 0 as the test's: no buffer; a buffer and a record whose own rate is -0x70.
    let one = (game_addr(RECORDS_RVA) as u32, RECORD_LEN);
    let saved = capture(&[one]);
    let buffer = [0x5A5A_5A5Au32, 0, 0, 0, 0, 0, 0, 0, 0, 11025, 0x13, 0x8000];
    for (has_buffer, own_volume, own_rate) in [
        (false, 0x30u32, 22050u32),
        (true, 0x30, BUFFER_RATE),
        (true, 0x50, OWN),
        (true, 0xFFFF_FFFF, 300_000),
    ] {
        let fill = || unsafe {
            let r = one.0 as *mut u32;
            r.add(2).write(if has_buffer {
                buffer.as_ptr() as u32
            } else {
                0
            });
            r.add(10).write(own_volume);
            r.add(12).write(own_rate);
        };
        for volume in TEST_VOLUMES {
            for rate in TEST_RATES {
                case(&mut t, 0, volume, rate, &fill);
            }
        }
    }
    restore(&[one], &saved);
    t.finish()
}
