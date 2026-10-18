//! `input::hat_direction` (0x408992): the device's hat (or buttons standing for it) as a
//! code in the second byte: for a device (+4) whose hat kind (+0x418) is not 0, the state's
//! buttons 6, 7, 1, 2 (+0x35, +0x36, +0x30, +0x31) as 0x100 .. 0x400, else its hat (+0x20)
//! straight on (0) 0x500, 270° (27000) 0x600, 180° 0x700, 90° 0xF00, else 0; without a
//! device or with kind 0, 0xE00 (none). (Code after the first `retn` that nothing reaches
//! reads buttons 1 and 2 as 1 and 2.)
//!
//! In:  esi = the device's record
//! Out: eax = the code; the rest kept; no flags read (the caller `or`s it)
//!
//! Caller: `input::read_device`. Self-test: the original and the Rust version on states by
//! lot and at each case, with and without a device, each hat kind.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::input::devices::{STATE_LEN, STATE_RVA, at, state_byte, test_record};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::hat_direction", rva(0x408992), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub fn hat_direction(record: u32) -> u32 {
    if at(record, 4) == 0 || at(record, 0x418) == 0 {
        return 0xE00;
    }
    for (button, code) in [(0x35, 0x100), (0x36, 0x200), (0x30, 0x300), (0x31, 0x400)] {
        if state_byte(button) != 0 {
            return code;
        }
    }
    let hat: u32 = unsafe { crate::framework::game_read(STATE_RVA + 0x20) };
    match hat {
        0 => 0x500,
        0x6978 => 0x600,
        0x4650 => 0x700,
        0x2328 => 0xF00,
        _ => 0,
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, hat_direction(r.esi)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept: [Region; 1] = [(game_addr(STATE_RVA) as u32, STATE_LEN)];
    let saved = capture(&kept);
    for (device, kind) in [(1u32, 1u32), (0, 1), (1, 0), (1, 7)] {
        let mut record = test_record(device * 3 + kind);
        record[4..8].copy_from_slice(&device.to_le_bytes());
        record[0x418..0x41C].copy_from_slice(&kind.to_le_bytes());
        let p = record.as_mut_ptr() as u32;
        // (the button pressed, the hat)
        for (button, hat) in [
            (None, 0u32),
            (None, 0x6978),
            (None, 0x4650),
            (None, 0x2328),
            (None, 0xFFFF_FFFF),
            (None, 4500),
            (Some(0x35), 0),
            (Some(0x36), 0x2328),
            (Some(0x30), 0),
            (Some(0x31), 0xFFFF),
            (Some(0x32), 0x4650),
        ] {
            check_state(
                &mut t,
                &format!("device {device}, hat kind {kind}, button {button:x?}, hat {hat:#x}"),
                call_original,
                Regs {
                    esi: p,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(game_addr(STATE_RVA) as *mut u8, 0, STATE_LEN);
                    game_write(STATE_RVA + 0x20, hat);
                    if let Some(b) = button {
                        game_write(STATE_RVA + b, 0x80u8);
                    }
                },
                &[],
                &rust,
            );
        }
        drop(record);
    }
    restore(&kept, &saved);
    t.finish()
}
