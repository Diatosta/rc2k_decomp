//! `input::hat_reading` (0x408968): the device's hat as an angle in 256ths: with a device
//! (+4) whose hat kind (+0x418) is 0 and a hat not centred (the state's +0x20,
//! `dword_4F6A48`, its low word not 0xFFFF), the hat's hundredths of degrees x 256 / 36000
//! (`div` of 0x8CA0); else 0.
//!
//! In:  esi = the device's record
//! Out: eax = the angle; ecx = 0x8CA0 and edx the remainder after the `div`; no flags read
//!
//! Caller: `input::read_device` (its bits' top byte). Self-test: the original and the Rust
//! version on angles by lot and centred, with and without a device, each hat kind.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::input::devices::{STATE_RVA, test_record};
use crate::input::devices::{joy_state, record_bytes};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::hat_reading", rva(0x408968), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HAT_RVA: usize = STATE_RVA + 0x20; // dword_4F6A48

/// The Rust version (see the header): (eax, the `div`'s ecx and edx when it was reached).
pub fn hat_reading(record: u32) -> (u32, Option<(u32, u32)>) {
    // Converted: `rc2k_core::input::devices::hat_reading`.
    rc2k_core::input::devices::hat_reading(&record_bytes(record), &joy_state())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (eax, div) = hat_reading(r.esi);
    let left = Left::nothing().with(EAX, eax);
    Ok(match div {
        Some((ecx, edx)) => left.with(ECX, ecx).with(EDX, edx),
        None => left,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept: [Region; 1] = [(crate::framework::game_addr(HAT_RVA) as u32, 4)];
    let saved = capture(&kept);
    for (device, kind) in [(1u32, 0u32), (0, 0), (1, 1)] {
        let mut record = test_record(device + kind);
        record[4..8].copy_from_slice(&device.to_le_bytes());
        record[0x418..0x41C].copy_from_slice(&kind.to_le_bytes());
        let p = record.as_mut_ptr() as u32;
        for hat in [
            0u32,
            9000,
            18000,
            27000,
            35999,
            0xFFFF,
            0xFFFF_FFFF,
            0x1234_5678,
            0x00FF_FFFF,
        ] {
            check_state(
                &mut t,
                &format!("device {device}, hat kind {kind}, hat {hat:#x}"),
                call_original,
                Regs {
                    esi: p,
                    ..Regs::SENTINEL
                },
                &|| unsafe { game_write(HAT_RVA, hat) },
                &[],
                &rust,
            );
        }
        drop(record);
    }
    restore(&kept, &saved);
    t.finish()
}
