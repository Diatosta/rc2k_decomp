//! `input::effect_found` (0x40953B): DirectInput's `EnumEffects` callback: each of the
//! device record's six effects (from +0x3C, 0xA4 bytes) whose kind (+4) is the found
//! effect's type (`DIEFFECTINFOA` +0x14, its low byte) takes it, unless it already has
//! one (+0xC 1) that is the GUID it wants (+8 points at it): +0xC 1, +0x20 .. +0x28 the
//! info's type, static and dynamic parameters, +0x10 its GUID. So each effect ends with
//! its wanted GUID when the device has it, else with the last of its type listed. Always
//! goes on (eax 1).
//!
//! In:  (stdcall) the `DIEFFECTINFOA`, the context (the device's record)
//!
//! Caller: DirectInput, from `input::begin_force`.

use crate::errors::BadInput;
use crate::input::guids::InputGuids;
use crate::input::types::{DiEffectInfo, InputDevice};

/// `effect_found` as DirectInput calls it: what Rust callers hand `EnumEffects`, the
/// device record the effects are listed for.
pub fn callback(guids: &InputGuids, info: &DiEffectInfo, record: &InputDevice) -> u32 {
    match effect_found(guids, info, record) {
        Ok(eax) => eax,
        Err(e) => {
            crate::input::devices::refused_here("input::effect_found", e);
            0
        }
    }
}

/// The Rust version: eax.
pub fn effect_found(
    guids: &InputGuids,
    info: &DiEffectInfo,
    record: &InputDevice,
) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::input::force::take_listed`.
    use rc2k_core::input::force::{Found, Listed, take_listed};
    let listed = Listed {
        guid: info.guid,
        eff_type: info.eff_type,
        static_params: info.static_params,
        dynamic_params: info.dynamic_params,
    };
    for e in &record.effects {
        let mut f = Found {
            found: e.found.get(),
            listed: Listed {
                guid: e.guid.read(),
                eff_type: e.eff_type.get(),
                static_params: e.static_params.get(),
                dynamic_params: e.dynamic_params.get(),
            },
        };
        // The wanted GUID is read only where the core asks for it.
        let failed = std::cell::Cell::new(None);
        let taken = take_listed(e.kind.get(), &mut f, &listed, || {
            crate::input::guids::guid_of(guids, e.wanted.get()).unwrap_or_else(|err| {
                failed.set(Some(err));
                [0; 16]
            })
        });
        if let Some(err) = failed.take() {
            return Err(err);
        }
        if !taken {
            continue;
        }
        e.found.set(f.found);
        e.eff_type.set(f.listed.eff_type);
        e.static_params.set(f.listed.static_params);
        e.dynamic_params.set(f.listed.dynamic_params);
        e.guid.write(&f.listed.guid);
    }
    Ok(1)
}
