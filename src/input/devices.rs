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

use crate::framework::{game_addr, game_read, rva};
use crate::service::harness::Region;

pub const RECORDS: [usize; 2] = [rva(0x4F6EF0), rva(0x4F73E8)];
pub const RECORD_LEN: usize = 0x4F8;
pub const KEYS_DOWN_RVA: usize = rva(0x4F6ADC); // g_keysDown: a byte a key, 0x80 down
pub const KEYS: usize = 0x100;
pub const STATE_RVA: usize = rva(0x4F6A28); // dword_4F6A28: the device's state, 0x50 bytes
pub const STATE_LEN: usize = 0x50;

/// A dword of the record at `record`.
pub fn at(record: u32, offset: u32) -> u32 {
    unsafe { ((record.wrapping_add(offset)) as *const u32).read_unaligned() }
}

/// Writes a dword of the record at `record`.
pub fn put(record: u32, offset: u32, v: u32) {
    unsafe { ((record.wrapping_add(offset)) as *mut u32).write_unaligned(v) }
}

/// A byte of the device's state.
pub fn state_byte(offset: usize) -> u8 {
    unsafe { game_read(STATE_RVA + offset) }
}

/// Whether key `key` is down.
pub fn key_down(key: u32) -> bool {
    unsafe { game_read::<u8>(KEYS_DOWN_RVA + (key as usize & 0xFF)) & 0x80 != 0 }
}

/// The key table and the device's state, as regions.
pub fn shared_regions() -> [Region; 2] {
    [
        (game_addr(KEYS_DOWN_RVA) as u32, KEYS),
        (game_addr(STATE_RVA) as u32, STATE_LEN),
    ]
}

/// A record of the test's: 0x4F8 bytes of a pattern.
pub fn test_record(seed: u32) -> Vec<u8> {
    (0..RECORD_LEN as u32)
        .map(|n| (n.wrapping_mul(0x9D) ^ seed.wrapping_mul(0x3B)) as u8)
        .collect()
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
            [a, b] => {
                let f: extern "system" fn(u32, u32, u32) -> u32 = std::mem::transmute(f);
                f(object, a, b)
            }
            _ => panic!("com_call: {} arguments", args.len()),
        }
    }
}
