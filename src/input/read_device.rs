//! A device's four readings (steering, brake, throttle, the bits) from its bindings (record
//! layout: `input/devices.rs`), after polling it. Each of steering left / right, throttle and
//! brake is its key down, or its axis (times its sign, below 0 made 0), or a button, "all of
//! it" being 0xFFFFFF00 for left and 0x100 for the others, setting bit 0x1000 (steering),
//! 0x4000 (throttle) or 0x2000 (brake); the steering is left plus right; on
//! a digital device every reading not 0 is made all of it. Eight buttons become bits 1 .. 0x80
//! and the hat the top two bytes of the bits. Refuses (before anything) a bound key past 0xFF
//! and an axis's place past the record.

use crate::errors::Refused;
use crate::input::devices::{joy_state, key_table, record_bytes};
use crate::input::store::Devices;
use crate::input::types::InputDevice;

/// The device `record` read (see the header). Converted: `rc2k_core::input::devices::read_device`; the bindings are checked before the poll,
/// the record read again after it.
pub fn read_device(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
    use rc2k_core::input::devices;
    devices::check_bindings(&record_bytes(record))?;
    { crate::input::poll_device::poll_device(devices, record) }?;
    let Some(r) = devices::read_device(
        &record_bytes(record),
        &key_table(devices),
        &joy_state(devices),
    )?
    else {
        return Ok(());
    };
    record.readings[0].set(r.steering);
    record.readings[1].set(r.brake);
    record.readings[2].set(r.throttle);
    record.bits.set(r.bits);
    Ok(())
}
