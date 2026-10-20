//! `sound::sample_duration` (0x40BA0B): how long a sample plays at a rate, in ms.
//!
//! In:  eax the sample (`g_sampleRecords`), edx the rate (-0x6F the record's +0x30, then
//!      -0x70 the buffer's +0x24, held to 100 .. 200000: `sound/sample_settings.rs`)
//! Out: eax the milliseconds; everything else preserved
//! A sample without a buffer (+8 null): 0; a looping one (+0x14 is 1): 0x7FFFFFFF.
//! Else the buffer's +0x2C (its length) x 1000 (`imul`, the low 32 bits) over the rate
//! shifted right by the buffer's +0x28 & 0xF (unsigned `div`).
//! Three callers (the race sounds' turn). Refuses a sample past the records and a rate
//! that the shift makes 0 (the original's `div` traps).
//! Shadow mode: both, the registers compared (`dispatch_state`, nothing written).
//! Self-test: every record with a set of rates, and record 0 as the test's (put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use crate::sound::sample_settings::{
    BUFFER_RATE, OWN, RECORD_LEN, RECORDS, RECORDS_RVA, TEST_RATES, loaded, record,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::sample_duration", rva(0x40BA0B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn sample_duration(sample: u32, rate: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::samples::duration_ms`.
    let record = record(sample)?;
    Ok(rc2k_core::samples::duration_ms(
        rate,
        loaded(record).as_ref(),
    )?)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, sample_duration(r.eax, r.edx)?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let case = |t: &mut SelfTest, sample: u32, rate: u32, fill: &dyn Fn()| {
        fill();
        // The original traps where the Rust version refuses: not run there.
        if sample_duration(sample, rate).is_err() {
            return;
        }
        check_state(
            t,
            &format!("sample {sample:#x}, rate {rate:#x}"),
            call_original,
            Regs {
                eax: sample,
                edx: rate,
                ..Regs::SENTINEL
            },
            fill,
            &[],
            &rust,
        );
    };
    for sample in 0..RECORDS {
        for rate in TEST_RATES {
            case(&mut t, sample, rate, &|| {});
        }
    }
    let one = (game_addr(RECORDS_RVA) as u32, RECORD_LEN);
    let saved = capture(&[one]);
    for (has_buffer, looping, own_rate, shift, length) in [
        (false, 0u32, 22050u32, 0u32, 0x1000u32),
        (true, 1, 22050, 0, 0x1000),
        (true, 0, BUFFER_RATE, 0x11, 0x0123_4567),
        (true, 0, OWN, 0x3, 0x10_0000),
        (true, 0, 300_000, 0x1F, 0x8000),
        (true, 2, 44100, 0, 0xFFFF_FFFF),
    ] {
        let buffer = [0u32, 0, 0, 0, 0, 0, 0, 0, 0, 11025, shift, length];
        let fill = || unsafe {
            let r = one.0 as *mut u32;
            r.add(2).write(if has_buffer {
                buffer.as_ptr() as u32
            } else {
                0
            });
            r.add(5).write(looping);
            r.add(12).write(own_rate);
        };
        for rate in TEST_RATES {
            case(&mut t, 0, rate, &fill);
        }
    }
    restore(&[one], &saved);
    t.finish()
}
