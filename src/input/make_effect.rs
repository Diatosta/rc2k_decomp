//! `input::make_effect` (0x40992F): one of a device record's six force-feedback effects
//! made: an object it had given back (`input::release_effect`); its `DIEFFECT` (+0x34,
//! 0x34 bytes: cartesian, by object offsets, for ever, gain 10000, no trigger, one axis, x
//! (+0x80; +0x84 4), direction 0 (+0x88), no envelope (one of 0x14 bytes at +0x90, all 0,
//! not linked)) and its type-specific parameters (+0x68) by its kind (+4): 1 a constant
//! force of 10000; 2 a ramp from 10000 to 10000; 3 a periodic one (magnitude 10000, offset
//! 0, phase 0, period 50000); 4 a condition (offset 0, both coefficients and saturations
//! 10000, dead band 0); others none (no size, no pointer); then its function (+0) called
//! with eax 1; and with an effect found for it (+0xC 1, `input::effect_found`) its object
//! made (`CreateEffect` of the record's device (+4) with the GUID found (+0x10), into
//! +0x30; its result not looked at).
//!
//! In:  esi = the device's record, edi = the effect, ecx = its number (for its function)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::begin_force` (each of the six). The effect's function is
//! one of `input::effect_N_change` (eax 1, the other registers as here), the methods the
//! objects' own.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::types::{ForceEffect, InputDevice};
use crate::pointers::GamePtr;

/// The effect's `DIEFFECT` and parameters, as the original writes them (in its order).
pub fn fill_parameters(e: &ForceEffect) {
    // Converted: `rc2k_core::input::force` (the figures, `type_specific`).
    use rc2k_core::input::force as k;
    let d = &e.effect;
    d.size.set(k::EFFECT_SIZE);
    d.flags.set(k::EFFECT_FLAGS);
    d.duration.set(k::FOR_EVER);
    d.sample_period.set(0);
    d.gain.set(k::FULL_GAIN);
    d.trigger_button.set(k::NO_BUTTON);
    d.trigger_repeat_interval.set(0);
    d.axes_count.set(k::AXES_COUNT);
    // the pointers the `DIEFFECT` holds are game addresses of the effect's own arrays
    d.axes.set(GamePtr::to(&e.axes).addr());
    e.axes[0].set(k::AXES[0]);
    e.axes[1].set(k::AXES[1]);
    d.directions.set(GamePtr::to(&e.directions).addr());
    e.directions[0].set(k::DIRECTION);
    d.envelope.set(0);
    d.type_specific_size.set(0);
    d.type_specific.set(0);
    // an envelope of its size, all 0, not linked
    e.envelope.size.set(k::ENVELOPE_SIZE);
    e.envelope.attack_level.set(0);
    e.envelope.attack_time.set(0);
    e.envelope.fade_level.set(0);
    e.envelope.fade_time.set(0);
    let params = k::type_specific(e.kind.get());
    if params.is_empty() {
        return;
    }
    d.type_specific_size.set(4 * params.len() as u32);
    d.type_specific.set(GamePtr::to(&e.type_specific).addr());
    for (cell, &v) in e.type_specific.iter().zip(params) {
        cell.set(v);
    }
}

/// The device `record`'s effect `n`, `effect`, made (see the header).
pub fn make_effect(record: &InputDevice, effect: &ForceEffect, n: u32) -> Result<(), Refused> {
    let e = effect;
    if e.object.get() != 0 {
        crate::input::effect_objects::release_effect(e);
    }
    fill_parameters(e);
    // The effect's function with a change of 1, ecx the effect's number, edx 0.
    crate::input::effect_changes::effect_change_at(e.change.get(), record, e, 1, n, 0)?;
    if e.found.get() == 1 {
        // The GUID and the `DIEFFECT`'s parts, as DirectInput takes them; the object's
        // cell as `CreateEffect` leaves it.
        let made = crate::input::di_objects::create_effect(
            record.device.get(),
            &e.guid.read(),
            Some(&crate::input::effect_objects::parameters_of(e)),
        );
        crate::input::di_objects::store(&e.object, made);
    }
    Ok(())
}
