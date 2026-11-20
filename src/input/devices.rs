//! The players' input devices, as the input thread keeps them: two records of 0x4F8 bytes.
//! What is known of a record:
//! - +0 the controller's number, +4 its DirectInput device (0: none), +8 its kind (1 digital:
//!   every reading all or nothing; 2, 3), +0xC .. +0x20 its settings (+0x1A .. +0x1D which
//!   state dwords its axes are), +0x24 1 while acquired,
//!   +0x28 1 while its effects run, +0x3C six effects of 0xA4 bytes;
//! - +0x2C, +0x30, +0x34 its three axes and +0x38 its buttons (bits), copied from the device's
//!   state at each poll;
//! - +0x414 1 while it is read, +0x418 its hat's kind; from +0x41C twelve bindings of 0x10 bytes
//!   (key, axis place, sign, button bit): steering left and right, throttle, brake, then eight
//!   buttons;
//! - +0x4DC .. +0x4E8 its four readings, which `drive::device_axes` reads.

use crate::errors::BadInput;
use crate::input::store::Devices;
use crate::input::types::InputDevice;

/// The records: the two players' and the third's and fourth's.
pub fn records(devices: &Devices) -> &[InputDevice; 4] {
    &devices.records
}

/// The device's state, 0x50 bytes.
pub(super) const STATE_LEN: usize = 0x50;

/// A record's bytes, for the core (`rc2k_core::input::devices`).
#[cfg(test)]
pub(super) fn record_bytes(record: &InputDevice) -> rc2k_core::input::devices::Record {
    record.to_bytes()
}

/// A refusal met inside a closure the core asks for a value (it has no failure to give):
/// the first one is kept, the closure answers 0, and the caller asks `finish` afterwards.
#[derive(Default)]
pub struct Deferred(std::cell::Cell<Option<BadInput>>);

impl Deferred {
    pub fn get(&self, read: Result<u32, BadInput>) -> u32 {
        read.unwrap_or_else(|e| {
            self.0.set(self.0.get().or(Some(e)));
            0
        })
    }

    pub fn finish(&self) -> Result<(), BadInput> {
        self.0.get().map_or(Ok(()), Err)
    }
}

/// What the program does with a refusal where a Windows callback leaves no way to hand it
/// on: it ends, saying which function refused and why.
pub(super) fn refused_here(name: &str, e: impl Into<crate::errors::Refused>) {
    crate::fatal::raise::refused(name, e);
}

/// The key table and the device's state, for the core.
pub fn key_table(devices: &Devices) -> rc2k_core::input::devices::KeyTable {
    devices.key_table()
}

pub(super) fn joy_state(devices: &Devices) -> rc2k_core::input::devices::State {
    let mut state = [0u8; STATE_LEN];
    for (n, dword) in devices.device_state.iter().enumerate() {
        state[4 * n..4 * n + 4].copy_from_slice(&dword.get().to_le_bytes());
    }
    state
}
