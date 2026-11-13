//! A device's force feedback set up, when its controller has it: the self-centring or the gain
//! (strength x 100) set, the device's effects listed and the six made. Each failure is a fatal
//! error. The controller's number is not checked.

use crate::errors::Refused;
use crate::input::di_harness::entry_of;
use crate::input::di_objects::{Property, set_property};

use crate::input::di_harness::fatal;
use crate::input::owner::Input;
use crate::input::types::{DiPropDword, DiPropHeader, InputDevice};
use rc2k_core::input::acquire::gain;
use std::mem::size_of;

const USING_TEXT_RVA: &std::ffi::CStr = c"INP: using force feedback"; // aInpUsingForceF

/// The `DIPROPDWORD` for the whole device with `data`.
pub fn dword_property(data: u32) -> DiPropDword {
    DiPropDword {
        header: DiPropHeader {
            size: size_of::<DiPropDword>() as u32,
            header_size: size_of::<DiPropHeader>() as u32,
            object: 0,
            how: 0,
        },
        data,
    }
}

/// See the header.
pub fn begin_force(input: &Input, record: &InputDevice) -> Result<(), Refused> {
    if entry_of(&input.devices, record)?.force_feedback.get() != 1 {
        return Ok(());
    }
    crate::logfile::log_write::log_nul(&USING_TEXT_RVA, true, 2);
    let device = record.device.get();
    let Some(gain) = gain(record.strength.get()) else {
        set_property(device, Property::AutoCenter(&dword_property(1)));
        return Ok(());
    };
    if set_property(device, Property::AutoCenter(&dword_property(0))) != 0 {
        return Err(fatal(0x8023_8421).into());
    }
    if set_property(device, Property::FfGain(&dword_property(gain))) != 0 {
        return Err(fatal(0x8023_8422).into());
    }
    crate::input::di_objects::enum_effects(
        device,
        |info| crate::input::effect_found::callback(&input.guids, info, record),
        0,
    );
    for (n, effect) in record.effects.iter().enumerate() {
        crate::input::make_effect::make_effect(&input.devices, record, effect, n as u32)?;
    }
    record.effects_on.set(1);
    Ok(())
}
