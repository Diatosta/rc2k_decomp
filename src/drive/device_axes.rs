//! `drive::device_axes` (0x409430): an input device's four readings
//! (`g_inputDevices[esi]`, +0x4DC .. +0x4E8: into ecx, edx, eax, ebx, as
//! `input_device_read` 0x40930D, inlined), with `dword_4F6900` set the first (eax)
//! squared / 256 with its sign kept.
//!
//! In:  esi the device
//! Out: eax, ebx, ecx, edx the readings; esi kept; no flags read
//!
//! Caller: `drive::car_input_words`.
//! Refuses a device past 7 or a null one.
//! Self-test: the original and the Rust version on a test device (`g_inputDevices[0]`
//! pointed at it), readings by lot, the curve on and off.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::drive::controls::device_curve;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::device_axes", rva(0x409430), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DEVICES: usize = 0x4F6EE8; // g_inputDevices

/// The Rust version: [eax, ebx, ecx, edx].
// Converted: `rc2k_core::drive::controls::device_curve`.
pub unsafe fn device_axes(device: u32) -> Result<[u32; 4], Refused> {
    if device > 7 {
        return Err(BadInput("a device past 7").into());
    }
    let d = g(DEVICES + 4 * device as usize);
    if d == 0 {
        return Err(BadInput("a null input device").into());
    }
    let rd = |o: u32| unsafe { ((d + o) as *const u32).read_unaligned() };
    let (ecx, edx, mut eax, ebx) = (rd(0x4DC), rd(0x4E0), rd(0x4E4), rd(0x4E8));
    if g(0x4F6900) != 0 {
        eax = device_curve(eax);
    }
    Ok([eax, ebx, ecx, edx])
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [eax, ebx, ecx, edx] = unsafe { device_axes(r.esi) }?;
    Ok(Left::nothing()
        .with(EAX, eax)
        .with(EBX, ebx)
        .with(ECX, ecx)
        .with(EDX, edx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [(a(DEVICES), 4), (a(0x4F6900), 4)];
    let saved = capture(&all);
    let mut device = vec![0u32; 0x500 / 4];
    let at = device.as_mut_ptr() as u32;
    for case in 0..8u32 {
        let set = || {
            p(DEVICES, at);
            p(0x4F6900, case % 2);
            for k in 0..4u32 {
                let v = (case.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x7F4A_7C15)) as i32
                    >> (case % 4 * 4);
                unsafe { ((at + 0x4DC + 4 * k) as *mut u32).write(v as u32) };
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                esi: 0,
                ..Regs::SENTINEL
            },
            &set,
            &[(at + 0x4DC, 16)],
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
