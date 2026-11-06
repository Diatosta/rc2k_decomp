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
use crate::input::di_harness::{CAPS_LEN, CONTROLLERS};
use crate::input::types::{Controller, DiDeviceInstance};
use crate::pointers::GamePtr;
use crate::system::owner::system;

/// Looks the joystick over and fills the entry: false at the first failure.
fn look_over(instance: &DiDeviceInstance, entry: &'static Controller) -> Result<bool, BadInput> {
    use crate::input::di_objects as di;
    let input = crate::input::owner::input().devices.direct_input.get();
    let made = di::create_device(input, &instance.instance);
    if di::store(&crate::input::owner::input().devices.new_joystick, made) != 0 {
        return Ok(false);
    }
    let device = crate::input::owner::input().devices.new_joystick.get();
    let made = di::query_interface(device, &crate::input::owner::input().guids.device2.read());
    if di::store(&crate::input::owner::input().devices.new_joystick2, made) != 0 {
        return Ok(false);
    }
    let device2 = crate::input::owner::input().devices.new_joystick2.get();
    if di::set_data_format(device2) != 0 {
        return Ok(false);
    }
    if di::set_cooperative_level(device2, system().session.machine.window.get(), 5) != 0 {
        return Ok(false);
    }
    let Ok(caps) = di::get_capabilities(device2, CAPS_LEN as u32) else {
        return Ok(false);
    };
    di::enum_objects(
        device2,
        |object| crate::input::axis_found::callback(object, entry),
        0,
    );
    let dev_type = instance.dev_type;
    let guid = instance.instance;
    entry.dev_type.set(dev_type);
    entry.instance.write(&guid);
    // Converted: `rc2k_core::input::joystick::{name_prefix, found}`.
    use rc2k_core::input::joystick::{NAME_ROOM, name_prefix};
    let count = crate::input::owner::input().devices.controller_count.get();
    let name = name_prefix(count);
    let at_name = GamePtr::to(&entry.name);
    at_name.write_bytes(&name, "a controller's name")?;
    use crate::strings::copy_printable::KEEP_CASE; // dword_4E0144
    use std::sync::atomic::Ordering::Relaxed;
    let kept = KEEP_CASE.atomic().swap(0, Relaxed);
    let to = at_name.offset(name.len() as u32);
    // room 0x19 is never refused
    if let Ok(printable) =
        crate::strings::copy_printable::printable_from(&instance.instance_name, NAME_ROOM)
    {
        to.write_bytes(&printable, "a controller's name")?;
    }
    KEEP_CASE.set(kept);
    let f = rc2k_core::input::joystick::found(
        caps.flags,
        caps.axes,
        caps.buttons,
        caps.hats,
        crate::input::owner::input().devices.no_force_feedback.get(),
    );
    entry.caps_flags.set(f.caps_flags);
    entry.axes.set(f.axes);
    entry.buttons.set(f.buttons);
    entry.hats.set(f.hats);
    entry.attached.set(f.attached);
    if let Some(force) = f.force_feedback {
        entry.force_feedback.set(force);
    }
    crate::input::owner::input()
        .devices
        .controller_count
        .set(count.wrapping_add(1));
    Ok(true)
}

/// The Rust version: eax.
/// `joystick_found` as DirectInput calls it: what Rust callers hand `EnumDevices`.
pub fn callback(instance: &DiDeviceInstance) -> u32 {
    match joystick_found(instance) {
        Ok(eax) => eax,
        Err(e) => {
            crate::input::devices::refused_here("input::joystick_found", e);
            0
        }
    }
}

pub fn joystick_found(instance: &DiDeviceInstance) -> Result<u32, BadInput> {
    let count = crate::input::owner::input().devices.controller_count.get();
    if count >= CONTROLLERS {
        return Ok(0);
    }
    let entry = &crate::input::owner::input().devices.controllers[count as usize];
    look_over(instance, entry)?;
    for cell in [
        &crate::input::owner::input().devices.new_joystick2,
        &crate::input::owner::input().devices.new_joystick,
    ] {
        let object = cell.get();
        if object != 0 {
            crate::input::di_objects::release(object);
            cell.set(0);
        }
    }
    Ok(1)
}
