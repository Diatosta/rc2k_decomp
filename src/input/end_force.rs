//! A device's force feedback ended, when its controller has it: the self-centring turned on
//! (the result not looked at), the six effects' objects given back and each effect's `found`
//! set to 0. The record's `effects_on` is left: `input::release_device` clears
//! it. The controller's number is not checked.

use crate::input::di_harness::entry_of;
use crate::input::store::Devices;
use crate::input::types::InputDevice;

/// See the header.
pub(super) fn end_force(
    devices: &Devices,
    record: &InputDevice,
) -> Result<(), crate::errors::BadInput> {
    if entry_of(devices, record)?.force_feedback.get() != 1 {
        return Ok(());
    }
    let prop = crate::input::begin_force::dword_property(1);
    crate::input::di_objects::set_property(
        record.device.get(),
        crate::input::di_objects::Property::AutoCenter(&prop),
    );
    for e in &record.effects {
        crate::input::effect_objects::release_effect(e);
        e.found.set(0);
    }
    Ok(())
}
