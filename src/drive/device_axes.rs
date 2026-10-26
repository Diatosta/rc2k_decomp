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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, Regs};
use crate::physics::flow::g;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::controls::device_curve;

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
