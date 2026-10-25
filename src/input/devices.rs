//! The players' input devices, as the input thread (`sub_4078DA`, "INPUTTDB") keeps them:
//! two records of 0x4F8 bytes (`unk_4F6EF0`, `unk_4F73E8`; `g_inputDevices` 0x4F6EE8 points
//! at them). What is known of a record (a reading from the code that uses it):
//! - +0 the controller's number, +4 its DirectInput device (0: none), +8 its kind (1
//!   digital: every reading all or nothing; 2, 3), +0xC .. +0x20 its settings, +0x1A ..
//!   +0x1D which state dwords its axes are, +0x24 1 while acquired, +0x28 1 while its
//!   effects run, +0x3C six effects of 0xA4 bytes;
//! - +0x2C, +0x30, +0x34 its three axes and +0x38 its buttons (bits), copied from the
//!   device's state (`dword_4F6A28`, 0x50 bytes, DirectInput's `DIJOYSTATE`) at each poll;
//! - +0x414 1 while it is read, +0x418 its hat's kind;
//! - from +0x41C twelve bindings of 0x10 bytes: the key (`g_keysDown`), the axis's place in
//!   the record (+0x2C, +0x30, +0x34; 0 none), its sign, a button's bit (0 none): steering
//!   left and right, throttle, brake, then eight buttons;
//! - +0x4DC .. +0x4E8 its four readings (steering, brake, throttle, the bits), which
//!   `drive::device_axes` reads.

use crate::input::store::DEVICES;

/// The two records' places (`g_inputRecord1`, `g_inputRecord2`).
pub fn record_addrs() -> [u32; 2] {
    DEVICES
        .records
        .each_ref()
        .map(|record| record.addr() as u32)
}

/// The device's state (`g_deviceState`), 0x50 bytes.
pub const STATE_LEN: usize = 0x50;

/// A dword of the record at `record`.
pub fn at(record: u32, offset: u32) -> u32 {
    unsafe { ((record.wrapping_add(offset)) as *const u32).read_unaligned() }
}

/// Writes a dword of the record at `record`.
pub fn put(record: u32, offset: u32, v: u32) {
    unsafe { ((record.wrapping_add(offset)) as *mut u32).write_unaligned(v) }
}

/// Whether key `key` is down.
/// A record's bytes, for the core (`rc2k_core::input::devices`).
pub fn record_bytes(record: u32) -> rc2k_core::input::devices::Record {
    unsafe { (record as *const rc2k_core::input::devices::Record).read_unaligned() }
}

/// The key table and the device's state, for the core.
pub fn key_table() -> rc2k_core::input::devices::KeyTable {
    DEVICES.key_table()
}

pub fn joy_state() -> rc2k_core::input::devices::State {
    let mut state = [0u8; STATE_LEN];
    for (n, dword) in DEVICES.device_state.iter().enumerate() {
        state[4 * n..4 * n + 4].copy_from_slice(&dword.get().to_le_bytes());
    }
    state
}

/// A method of a COM object (DirectInput's device): the function in slot `slot` (a byte
/// offset) of its table, `stdcall` with the object first; the method's result.
pub unsafe fn com_call(object: u32, slot: u32, args: &[u32]) -> u32 {
    unsafe {
        let table = (object as *const u32).read_unaligned();
        let f = ((table + slot) as *const u32).read_unaligned() as usize;
        match *args {
            [] => {
                let f: extern "system" fn(u32) -> u32 = std::mem::transmute(f);
                f(object)
            }
            [a] => {
                let f: extern "system" fn(u32, u32) -> u32 = std::mem::transmute(f);
                f(object, a)
            }
            [a, b] => {
                let f: extern "system" fn(u32, u32, u32) -> u32 = std::mem::transmute(f);
                f(object, a, b)
            }
            [a, b, c] => {
                let f: extern "system" fn(u32, u32, u32, u32) -> u32 = std::mem::transmute(f);
                f(object, a, b, c)
            }
            [a, b, c, d] => {
                let f: extern "system" fn(u32, u32, u32, u32, u32) -> u32 = std::mem::transmute(f);
                f(object, a, b, c, d)
            }
            _ => panic!("com_call: {} arguments", args.len()),
        }
    }
}
