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
//! Shadow mode runs the original alone. Self-test: the original and the Rust version on
//! DirectInput objects made in Rust (`input::di_harness`): the joystick and the wheel, a
//! device that is not there (`CreateDevice` fails), 0, 5, 15 and 16 controllers before,
//! the force-feedback switch both ways.

use crate::field;
use crate::framework::{RawHook, SelfTest, game_addr, rva};
use crate::input::devices::{at, put};
use crate::input::di_harness::{
    CAPS_LEN, CAPS_RVA, CONTROLLER_COUNT_RVA, CONTROLLER_LEN, CONTROLLERS, CONTROLLERS_RVA,
    DIRECT_INPUT_RVA, HWND_RVA, IID_DEVICE2_RVA, JOYSTICK_FORMAT_RVA, NEW_DEVICE_RVA,
    NEW_DEVICE2_RVA, a, call_stdcall2, compare, dispatch_callback, g, method, p, shared_regions,
    slot,
};
use crate::input::types::{Controller, DiDevCaps, DiDeviceInstance};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(instance: u32, context: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        instance,
        context,
        joystick_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "input::joystick_found",
    rva(0x408238),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

const AXIS_FOUND: usize = 0x4083F4; // input::axis_found
pub const NO_FORCE_RVA: usize = rva(0x4F68F4); // dword_4F68F4: force feedback turned off
pub const KEEP_CASE_RVA: usize = rva(0x4E0144); // dword_4E0144

/// Looks the joystick over and fills the entry: false at the first failure.
fn look_over(instance: u32, entry: u32) -> bool {
    let input = g(DIRECT_INPUT_RVA);
    let ok = |hr: u32| hr == 0;
    if !ok(method(
        input,
        slot::CREATE_DEVICE,
        &[
            instance + field!(DiDeviceInstance, instance),
            a(NEW_DEVICE_RVA),
            0,
        ],
    )) {
        return false;
    }
    let device = g(NEW_DEVICE_RVA);
    if !ok(method(
        device,
        slot::QUERY,
        &[a(IID_DEVICE2_RVA), a(NEW_DEVICE2_RVA)],
    )) {
        return false;
    }
    let device2 = g(NEW_DEVICE2_RVA);
    if !ok(method(
        device2,
        slot::SET_DATA_FORMAT,
        &[a(JOYSTICK_FORMAT_RVA)],
    )) {
        return false;
    }
    if !ok(method(
        device2,
        slot::SET_COOPERATIVE_LEVEL,
        &[g(HWND_RVA), 5],
    )) {
        return false;
    }
    p(CAPS_RVA, CAPS_LEN as u32);
    if !ok(method(device2, slot::GET_CAPABILITIES, &[a(CAPS_RVA)])) {
        return false;
    }
    let found = game_addr(rva(AXIS_FOUND)) as u32;
    method(device2, slot::ENUM_OBJECTS, &[found, entry, 0]);
    put(
        entry,
        field!(Controller, dev_type),
        at(instance, field!(DiDeviceInstance, dev_type)),
    );
    let guid = instance + field!(DiDeviceInstance, instance);
    let to = entry + field!(Controller, instance);
    unsafe { std::ptr::copy(guid as *const u8, to as *mut u8, 16) };
    // Converted: `rc2k_core::input::joystick::{name_prefix, found}`.
    use rc2k_core::input::joystick::{NAME_ROOM, name_prefix};
    let count = g(CONTROLLER_COUNT_RVA);
    let name = name_prefix(count);
    let at_name = entry + field!(Controller, name);
    unsafe { std::ptr::copy(name.as_ptr(), at_name as *mut u8, name.len()) };
    let kept = g(KEEP_CASE_RVA);
    p(KEEP_CASE_RVA, 0);
    let to = at_name as usize + name.len();
    // room 0x19 is never refused
    let _ = unsafe {
        crate::strings::copy_printable::copy_printable(
            (instance + field!(DiDeviceInstance, instance_name)) as *const u8,
            to as *mut u8,
            NAME_ROOM,
        )
    };
    p(KEEP_CASE_RVA, kept);
    let caps = |offset: u32| g(CAPS_RVA + offset as usize);
    let f = rc2k_core::input::joystick::found(
        caps(field!(DiDevCaps, flags)),
        caps(field!(DiDevCaps, axes)),
        caps(field!(DiDevCaps, buttons)),
        caps(field!(DiDevCaps, hats)),
        g(NO_FORCE_RVA),
    );
    put(entry, field!(Controller, caps_flags), f.caps_flags);
    put(entry, field!(Controller, axes), f.axes);
    put(entry, field!(Controller, buttons), f.buttons);
    put(entry, field!(Controller, hats), f.hats);
    put(entry, field!(Controller, attached), f.attached);
    if let Some(force) = f.force_feedback {
        put(entry, field!(Controller, force_feedback), force);
    }
    p(CONTROLLER_COUNT_RVA, count.wrapping_add(1));
    true
}

/// The Rust version: eax.
pub fn joystick_found(instance: u32, _context: u32) -> u32 {
    let count = g(CONTROLLER_COUNT_RVA);
    if count >= CONTROLLERS {
        return 0;
    }
    let entry = a(CONTROLLERS_RVA) + count * CONTROLLER_LEN;
    look_over(instance, entry);
    for at in [NEW_DEVICE2_RVA, NEW_DEVICE_RVA] {
        let object = g(at);
        if object != 0 {
            method(object, slot::RELEASE, &[]);
            p(at, 0);
        }
    }
    1
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let devices = crate::input::di_objects::test_devices();
    let world = crate::input::di_objects::TestWorld::begin(devices.clone(), Default::default());
    let original = ORIGINAL.load(Ordering::SeqCst);
    let mut regions = shared_regions();
    regions.push((a(NO_FORCE_RVA), 4));
    regions.push((a(KEEP_CASE_RVA), 4));
    let mut missing = devices[0].clone();
    missing.instance[0] ^= 0xFF;
    let instances: Vec<crate::input::types::DiDeviceInstance> = devices
        .iter()
        .chain([&missing])
        .map(crate::input::di_objects::device_instance)
        .collect();
    for (n, instance) in instances.iter().enumerate() {
        for count in [0u32, 5, 15, 16] {
            for no_force in [0u32, 1] {
                let i = instance as *const crate::input::types::DiDeviceInstance as u32;
                compare(
                    &mut t,
                    &format!("device {n}, {count} before, force feedback off {no_force}"),
                    &world,
                    &regions,
                    &|| {
                        p(DIRECT_INPUT_RVA, crate::input::di_objects::create_input());
                        p(NEW_DEVICE_RVA, 0);
                        p(NEW_DEVICE2_RVA, 0);
                        p(CONTROLLER_COUNT_RVA, count);
                        p(NO_FORCE_RVA, no_force);
                        p(KEEP_CASE_RVA, 1);
                        let table = a(CONTROLLERS_RVA) as *mut u8;
                        unsafe {
                            std::ptr::write_bytes(
                                table,
                                0xAA,
                                (CONTROLLERS * CONTROLLER_LEN) as usize,
                            )
                        };
                    },
                    &mut || format!("{}", unsafe { call_stdcall2(original, i, 0x1234) }),
                    &mut || format!("{}", joystick_found(i, 0x1234)),
                );
            }
        }
    }
    drop(world);
    t.finish()
}
