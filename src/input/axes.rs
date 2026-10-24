//! A controller's state and axes as the front end reads them (stream A, part F1,
//! 2026-10-18): the first device (`g_inputDevices`) holds +0x2C .. +0x38 four values the
//! poll left (x axis, y axis, z axis, the buttons' bits); the six axes now at
//! `dword_4F6A28` and as they were at `unk_4F8270`; the device's record (`dword_4F78E4` +
//! 0x50 x `unk_4F6EF0`) +0x14 the axes it has.

use crate::framework::{game_addr, game_read, rva};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// `input::device_state` (0x408A49): device esi's four values: (ecx, edx, eax, ebx) =
/// (+0x2C, +0x30, +0x34, +0x38).
pub unsafe fn device_state(device: u32) -> [u32; 4] {
    unsafe {
        std::array::from_fn(|i| ((device + 0x2C + 4 * i as u32) as *const u32).read_unaligned())
    }
}

/// `input::first_device_state` (0x409391): `device_state` of `g_inputDevices`.
pub unsafe fn first_device_state() -> [u32; 4] {
    unsafe { device_state(k(0x4F6EE8)) }
}

/// `first_device_state` where there is a device (`g_inputDevices` not 0), else refused:
/// the calls by address (`routes.rs`; the original would read through 0).
pub fn first_device_state_checked() -> Result<[u32; 4], crate::errors::Refused> {
    if k(0x4F6EE8) == 0 {
        return Err(crate::errors::BadInput("no input device").into());
    }
    Ok(unsafe { first_device_state() })
}

/// What `input::axis_moved` leaves: the carry (none), eax.
pub struct Moved {
    pub carry: bool,
    pub eax: u32,
}

/// `input::axis_moved` (0x40939F): with eax 0, after a sleep of 250 ms (`system::sleep`) the
/// axes' rest values taken (`dword_4F6A28` -> `unk_4F8270`), carry. Else the first of the
/// device's axes (its record's +0x14 bits) moved more than 0x80 from rest: eax its number,
/// +0x10 when the way was up; carry when none (eax the last difference).
pub unsafe fn axis_moved(eax: u32) -> Moved {
    unsafe {
        let now = game_addr(rva(0x4F6A28)) as *const u32;
        let rest = game_addr(rva(0x4F8270)) as *mut u32;
        if eax == 0 {
            crate::system::sleep::sleep(250);
            std::ptr::copy(now, rest, 6);
            return Moved { carry: true, eax };
        }
        // Converted: `rc2k_core::input::devices::axis_moved`.
        let device = k(0x4F6EF0).wrapping_mul(0x50);
        let axes = ((game_addr(rva(0x4F78E4)) as u32).wrapping_add(device) + 0x14) as *const u32;
        let now = (now as *const [u32; 6]).read_unaligned();
        let rest = (rest as *const [u32; 6]).read_unaligned();
        match rc2k_core::input::devices::axis_moved(&now, &rest, axes.read_unaligned()) {
            Ok(axis) => Moved {
                carry: false,
                eax: axis,
            },
            Err(last) => Moved {
                carry: true,
                eax: last,
            },
        }
    }
}
