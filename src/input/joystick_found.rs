//! `input::joystick_found` (0x408238): DirectInput's `EnumDevices` callback: with fewer than
//! 16 controllers (`g_controllerCount`) the found joystick is looked at through a device
//! made for it (`CreateDevice` with its instance GUID into `dword_4F6910`, its
//! `IDirectInputDevice2A` into `dword_4F6914`, the joystick data format, the cooperative
//! level 5 (exclusive, background) for `hWnd`, `GetCapabilities` into `dword_4F6A78`, its
//! axes into the entry's +0x14 bits (`EnumObjects` with `input::axis_found`)), and the next
//! entry of `g_controllers` (0x50 bytes) filled: +0 the capabilities' flags, +4 the device's
//! type, +8 its axes, +0xC buttons, +0x10 hats, +0x18 1 if attached, +0x1C 1 if it has force
//! feedback (only when `dword_4F68F4` is 0: the configuration's switch turns it off), +0x20
//! its instance GUID, +0x30 its name as "(n) name" (n the count + 1; the name through
//! `strings::copy_printable`, 0x19 room, `dword_4E0144` 0 meanwhile: lower case), and the
//! count one more. Any failure leaves the entry and the count. Both objects are given back
//! in the end, whatever happened. With 16 the listing stops (eax 0), else it goes on (1).
//!
//! In:  (stdcall) the `DIDEVICEINSTANCEA`, the context (not read)
//! Out: eax = 1 go on, 0 stop; ebx, esi, edi, ebp kept; `ret 8`
//!
//! Caller: DirectInput, from `input::list_joysticks`. The methods the objects' own.

use crate::errors::BadInput;
use crate::field;
use crate::input::devices::put;
use crate::input::di_harness::{
    CAPS_LEN, CONTROLLER_LEN, CONTROLLERS, JOYSTICK_FORMAT_RVA, a, method, slot,
};
use crate::input::guids::INPUT_GUIDS;
use crate::input::store::DEVICES;
use crate::input::types::{Controller, DiDevCaps, DiDeviceInstance};
use crate::pointers::GamePtr;
use crate::session::MACHINE;

/// Looks the joystick over and fills the entry: false at the first failure.
fn look_over(instance: u32, entry: u32) -> Result<bool, BadInput> {
    let input = DEVICES.direct_input.get();
    let ok = |hr: u32| hr == 0;
    if !ok(method(
        input,
        slot::CREATE_DEVICE,
        &[
            instance + field!(DiDeviceInstance, instance),
            DEVICES.new_joystick.addr(),
            0,
        ],
    )) {
        return Ok(false);
    }
    let device = DEVICES.new_joystick.get();
    if !ok(method(
        device,
        slot::QUERY,
        &[INPUT_GUIDS.device2.addr(), DEVICES.new_joystick2.addr()],
    )) {
        return Ok(false);
    }
    let device2 = DEVICES.new_joystick2.get();
    if !ok(method(
        device2,
        slot::SET_DATA_FORMAT,
        &[a(JOYSTICK_FORMAT_RVA)],
    )) {
        return Ok(false);
    }
    if !ok(method(
        device2,
        slot::SET_COOPERATIVE_LEVEL,
        &[MACHINE.window.get(), 5],
    )) {
        return Ok(false);
    }
    DEVICES.joystick_caps.set_u32(0, CAPS_LEN as u32);
    if !ok(method(
        device2,
        slot::GET_CAPABILITIES,
        &[DEVICES.joystick_caps.addr()],
    )) {
        return Ok(false);
    }
    let found = crate::pointers::game_address(crate::input::axis_found::callback as *const ());
    method(device2, slot::ENUM_OBJECTS, &[found, entry, 0]);
    // SAFETY: `instance` is the `DIDEVICEINSTANCEA` the emulation lists (a local of its own,
    // no allocation of the game's), alive for this call.
    let (dev_type, guid) = unsafe {
        (
            (crate::pointers::host(instance + field!(DiDeviceInstance, dev_type)) as *const u32)
                .read_unaligned(),
            (crate::pointers::host(instance + field!(DiDeviceInstance, instance))
                as *const [u8; 16])
                .read_unaligned(),
        )
    };
    put(entry, field!(Controller, dev_type), dev_type)?;
    GamePtr::new(entry)
        .offset(field!(Controller, instance))
        .write(guid, "a controller's instance GUID")?;
    // Converted: `rc2k_core::input::joystick::{name_prefix, found}`.
    use rc2k_core::input::joystick::{NAME_ROOM, name_prefix};
    let count = DEVICES.controller_count.get();
    let name = name_prefix(count);
    let at_name = entry + field!(Controller, name);
    GamePtr::new(at_name).write_bytes(&name, "a controller's name")?;
    use crate::strings::copy_printable::KEEP_CASE; // dword_4E0144
    use std::sync::atomic::Ordering::Relaxed;
    let kept = KEEP_CASE.swap(0, Relaxed);
    let to = at_name as usize + name.len();
    // room 0x19 is never refused
    let _ = unsafe {
        crate::strings::copy_printable::copy_printable(
            crate::pointers::host(instance + field!(DiDeviceInstance, instance_name)) as *const u8,
            crate::pointers::host_at(to),
            NAME_ROOM,
        )
    };
    KEEP_CASE.store(kept, Relaxed);
    let caps = |offset: u32| DEVICES.joystick_caps.get_u32(offset as usize);
    let f = rc2k_core::input::joystick::found(
        caps(field!(DiDevCaps, flags)),
        caps(field!(DiDevCaps, axes)),
        caps(field!(DiDevCaps, buttons)),
        caps(field!(DiDevCaps, hats)),
        DEVICES.no_force_feedback.get(),
    );
    put(entry, field!(Controller, caps_flags), f.caps_flags)?;
    put(entry, field!(Controller, axes), f.axes)?;
    put(entry, field!(Controller, buttons), f.buttons)?;
    put(entry, field!(Controller, hats), f.hats)?;
    put(entry, field!(Controller, attached), f.attached)?;
    if let Some(force) = f.force_feedback {
        put(entry, field!(Controller, force_feedback), force)?;
    }
    DEVICES.controller_count.set(count.wrapping_add(1));
    Ok(true)
}

/// The Rust version: eax.
/// `joystick_found` as DirectInput calls it: what Rust callers hand `EnumDevices`.
pub extern "system" fn callback(instance: u32, context: u32) -> u32 {
    match joystick_found(instance, context) {
        Ok(eax) => eax,
        Err(e) => {
            crate::input::devices::refused_here("input::joystick_found", e);
            0
        }
    }
}

pub fn joystick_found(instance: u32, _context: u32) -> Result<u32, BadInput> {
    let count = DEVICES.controller_count.get();
    if count >= CONTROLLERS {
        return Ok(0);
    }
    let entry = DEVICES.controllers[0].addr() + count * CONTROLLER_LEN;
    look_over(instance, entry)?;
    for cell in [&DEVICES.new_joystick2, &DEVICES.new_joystick] {
        let object = cell.get();
        if object != 0 {
            method(object, slot::RELEASE, &[]);
            cell.set(0);
        }
    }
    Ok(1)
}
