//! `input::begin_force` (0x40945D): a device's force feedback set up, when its controller
//! has it (the controller's entry, `g_controllers` + 0x50 its number, +0x1C 1): "INP:
//! using force feedback" logged (level 2); the `DIPROPDWORD` (`dword_4F6AA4`: 0x14, 0x10,
//! the device, 0); with the record's strength (+0xC) 0 only the self-centring turned on
//! (`DIPROP_AUTOCENTER` 1); else the self-centring off and the gain the strength x 100
//! (`DIPROP_FFGAIN`), each failure a fatal error (0x80238421, 0x80238422), the device's
//! effects listed (`EnumEffects` with `input::effect_found`, the record as context), the
//! six made (`input::make_effect`, ecx the number, edi the effect) and +0x28 1 (effects
//! on).
//!
//! In:  esi = the device's record (+4 its DirectInput device)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::acquire_device`. The controller's number is not checked (as the
//! original's). The effects' functions through `make_effect`'s `part`; the methods the
//! device's own.

use crate::errors::Refused;
use crate::field;
use crate::framework::rva;
use crate::input::devices::{at, put};
use crate::input::di_harness::{EFFECT_LEN, EFFECTS_AT, entry_of, fatal, method, slot};
use crate::input::store::DEVICES;
use crate::input::types::{Controller, DiPropDword, DiPropHeader, InputDevice};
use crate::store::game_addr;
use rc2k_core::input::acquire::gain;
use std::mem::size_of;

const USING_TEXT_RVA: usize = rva(0x4DA5D8); // aInpUsingForceF
const DIPROP_FFGAIN: u32 = 7;
const DIPROP_AUTOCENTER: u32 = 9;

/// The `DIPROPDWORD` for the whole device with `data`: its address.
pub fn dword_property(data: u32) -> u32 {
    let prop = DEVICES.dword_property.addr() as u32;
    put(
        prop,
        field!(DiPropDword, header.size),
        size_of::<DiPropDword>() as u32,
    );
    put(
        prop,
        field!(DiPropDword, header.header_size),
        size_of::<DiPropHeader>() as u32,
    );
    put(prop, field!(DiPropDword, header.object), 0);
    put(prop, field!(DiPropDword, header.how), 0);
    put(prop, field!(DiPropDword, data), data);
    prop
}

/// The Rust version (see the header).
pub fn begin_force(record: u32) -> Result<(), Refused> {
    let entry = entry_of(record);
    if at(entry, field!(Controller, force_feedback)) != 1 {
        return Ok(());
    }
    unsafe {
        crate::logfile::log_write::log_write(game_addr(USING_TEXT_RVA) as *const u8, true, 2)
    };
    let device = at(record, field!(InputDevice, device));
    // Converted: `rc2k_core::input::acquire::gain`.
    let Some(gain) = gain(at(record, field!(InputDevice, strength))) else {
        method(
            device,
            slot::SET_PROPERTY,
            &[DIPROP_AUTOCENTER, dword_property(1)],
        );
        return Ok(());
    };
    if method(
        device,
        slot::SET_PROPERTY,
        &[DIPROP_AUTOCENTER, dword_property(0)],
    ) != 0
    {
        return Err(fatal(0x8023_8421).into());
    }
    let gain = dword_property(gain);
    if method(device, slot::SET_PROPERTY, &[DIPROP_FFGAIN, gain]) != 0 {
        return Err(fatal(0x8023_8422).into());
    }
    let found = crate::input::effect_found::callback as unsafe extern "system" fn(u32, u32) -> u32
        as usize as u32;
    method(device, slot::ENUM_EFFECTS, &[found, record, 0]);
    for n in 0..6 {
        let effect = record.wrapping_add(EFFECTS_AT + n * EFFECT_LEN);
        crate::input::make_effect::make_effect(record, effect, n)?;
    }
    put(record, field!(InputDevice, effects_on), 1);
    Ok(())
}
