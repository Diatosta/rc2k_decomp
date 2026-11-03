//! `input::end_force` (0x4095AE): a device's force feedback ended, when its controller has
//! it (the controller's entry, `g_controllers` + 0x50 its number, +0x1C 1): the
//! self-centring turned on (`DIPROP_AUTOCENTER` 1 in `dword_4F6AA4`; its result not looked
//! at) and the six effects' objects given back (`input::release_effect`), each effect's
//! +0xC (found) 0. (+0x28, effects on, is left: `input::release_device` clears it.)
//!
//! In:  esi = the device's record (+4 its DirectInput device)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::release_device`. The controller's number is not checked (as the
//! original's).

use crate::field;
use crate::input::di_harness::controller_dword;
use crate::input::types::{Controller, InputDevice};

/// The Rust version (see the header).
pub fn end_force(record: &InputDevice) -> Result<(), crate::errors::BadInput> {
    if controller_dword(record, field!(Controller, force_feedback))? != 1 {
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
