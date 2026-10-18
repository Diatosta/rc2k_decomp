//! `input::clear_device_readings` (0x40883C): a device's state copies (+0x2C .. +0x38) and
//! readings (+0x4DC .. +0x4E4) 0, its bits reading +0x4E8 0xE00 (the hat centred).
//!
//! In:  esi = the device's record (`input/devices.rs`)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_408473` (an acquiring that failed), `sub_4087F3` (the release).
//! Self-test: the original and the Rust version on records of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::devices::{RECORD_LEN, put, test_record};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "input::clear_device_readings",
    rva(0x40883C),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub fn clear_device_readings(record: u32) {
    for offset in [0x2C, 0x30, 0x34, 0x38, 0x4DC, 0x4E0, 0x4E4] {
        put(record, offset, 0);
    }
    put(record, 0x4E8, 0xE00);
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    clear_device_readings(r.esi);
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [
        (r.esi.wrapping_add(0x2C), 0x10),
        (r.esi.wrapping_add(0x4DC), 0x10),
    ];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for seed in 0..3u32 {
        let mut record = test_record(seed);
        let start = record.clone();
        let p = record.as_mut_ptr() as u32;
        check_state(
            &mut t,
            &format!("record {seed}"),
            call_original,
            Regs {
                esi: p,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u8, RECORD_LEN) },
            &[(p, RECORD_LEN)],
            &rust,
        );
        drop(record);
    }
    t.finish()
}
