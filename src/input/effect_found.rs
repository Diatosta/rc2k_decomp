//! `input::effect_found` (0x40953B): DirectInput's `EnumEffects` callback: each of the
//! device record's six effects (from +0x3C, 0xA4 bytes) whose kind (+4) is the found
//! effect's type (`DIEFFECTINFOA` +0x14, its low byte) takes it, unless it already has
//! one (+0xC 1) that is the GUID it wants (+8 points at it): +0xC 1, +0x20 .. +0x28 the
//! info's type, static and dynamic parameters, +0x10 its GUID. So each effect ends with
//! its wanted GUID when the device has it, else with the last of its type listed. Always
//! goes on (eax 1).
//!
//! In:  (stdcall) the `DIEFFECTINFOA`, the context (the device's record)
//! Out: eax = 1; ebx, esi, edi, ebp kept; `ret 8`
//!
//! Caller: DirectInput, from `input::begin_force`.

use crate::errors::BadInput;
use crate::field;
use crate::input::types::{DiEffectInfo, InputDevice};
use crate::pointers::GamePtr;

/// `effect_found` as DirectInput calls it: what Rust callers hand `EnumEffects`; the
/// context is the device record's handle (`Devices::handle_of`).
pub fn callback(info: u32, context: u32) -> u32 {
    let Some(record) = crate::input::store::device_of(context) else {
        crate::input::devices::refused_here(
            "input::effect_found",
            BadInput("an effect listing's context that is no device record"),
        );
        return 0;
    };
    match effect_found(info, record) {
        Ok(eax) => eax,
        Err(e) => {
            crate::input::devices::refused_here("input::effect_found", e);
            0
        }
    }
}

/// A dword of the listed effect at `at`: DirectInput's emulation holds it in a local of its
/// own, which is no allocation of the game's (a slice when the callback takes one).
fn listed_dword(at: u32) -> u32 {
    crate::pointers::read_or_end(at, "a listed effect")
}

/// The Rust version: eax.
pub fn effect_found(info: u32, record: &InputDevice) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::input::force::take_listed`.
    use rc2k_core::input::force::{Found, Listed, take_listed};
    let guid = |at: u32| GamePtr::new(at).read::<[u8; 16]>("an effect's GUID");
    // `info` is the `DIEFFECTINFOA` the emulation lists, alive for this call.
    let listed = {
        Listed {
            guid: guid(info + field!(DiEffectInfo, guid))?,
            eff_type: listed_dword(info + field!(DiEffectInfo, eff_type)),
            static_params: listed_dword(info + field!(DiEffectInfo, static_params)),
            dynamic_params: listed_dword(info + field!(DiEffectInfo, dynamic_params)),
        }
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
            crate::input::guids::guid_of(e.wanted.get()).unwrap_or_else(|err| {
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
