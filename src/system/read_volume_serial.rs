//! `system::read_volume_serial` (0x403131): the current drive's serial number and volume
//! name (`GetVolumeInformationA` with no root), into the first 16 bytes of `g_pngInfo`
//! (0x93E8B0, a scratch place here): +0 the serial, +4 the name in 12 bytes, +0xF made 0.
//!
//! In:  esi = anything
//! Out: the original pushes esi and ends with `pop eax`, so eax = the entry's esi (the
//!      serial meant for eax is lost: `notes/bugs.md`), esi = 0x93E8B0. Worked: ebx, ecx,
//!      edx = the dwords at +4, +8, +0xC; carry clear, zero set if the serial is 0
//!      (`test eax, eax`). Failed: ebx, ecx, edx = 0, carry and zero set. edi, ebp kept.
//! One caller, `start` (0x40100B), before `math::stir_random`, which stirs in eax. In the
//! game this never gets a call (the hook DLL is loaded later); the standalone program needs
//! it. The Win32 call is made from Rust (it is the same call in both programs).
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::framework::{Regs, game_addr, rva};
use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationA;

// ---- game data ----

const VOLUME_RVA: usize = rva(0x93E8B0); // g_pngInfo: its first 16 bytes
// ---- Rust version ----

/// The registers and flags the original leaves (see the header).
pub unsafe fn read_volume_serial(entry: &Regs) -> Regs {
    let at = game_addr(VOLUME_RVA) as *mut u8;
    unsafe {
        let worked = GetVolumeInformationA(
            std::ptr::null(),
            at.add(4),
            12,
            at as *mut u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        ) != 0;
        let mut r = Regs {
            eax: entry.esi,
            esi: at as u32,
            ..*entry
        };
        if worked {
            // Converted: `rc2k_core::system::machine::volume_registers`.
            let info = &mut *(at as *mut [u8; 16]);
            let ([b, c, d], zero) = rc2k_core::system::machine::volume_registers(info);
            (r.ebx, r.ecx, r.edx) = (b, c, d);
            r.set_carry(false);
            r.set_zero(zero);
        } else {
            (r.ebx, r.ecx, r.edx) = (0, 0, 0);
            r.set_carry(true);
            r.set_zero(true);
        }
        r
    }
}
