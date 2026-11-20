//! Makes one of a device record's six force-feedback effects: the object it had is given back;
//! its `DIEFFECT` (+0x34) and its type-specific parameters (+0x68) are set by its kind (+4: 1
//! constant force, 2 ramp, 3 periodic, 4 condition, others none) at their default values; its
//! change function is called with the change 1; and with an effect found for it (+0xC 1) its
//! object is made (`CreateEffect`; the result is not looked at).

use crate::errors::Refused;
use crate::input::store::Devices;
use crate::input::types::{ForceEffect, InputDevice};

/// What a `DIEFFECT` pointer (`axes`, `directions`, `envelope`, `type_specific`) holds when
/// it points at the effect's own part: any value but 0 (nothing reads the address).
pub(super) const PART_SET: u32 = 1;

/// The effect's `DIEFFECT` and parameters.
fn fill_parameters(e: &ForceEffect) {
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
    // The pointers the `DIEFFECT` holds are `PART_SET`: only `effect_objects::parameters_of`
    // reads them, as set or not.
    d.axes.set(PART_SET);
    e.axes[0].set(k::AXES[0]);
    e.axes[1].set(k::AXES[1]);
    d.directions.set(PART_SET);
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
    d.type_specific.set(PART_SET);
    for (cell, &v) in e.type_specific.iter().zip(params) {
        cell.set(v);
    }
}

/// The device `record`'s effect `n`, `effect`, made (see the header).
pub(super) fn make_effect(
    devices: &Devices,
    record: &InputDevice,
    effect: &ForceEffect,
    n: u32,
) -> Result<(), Refused> {
    let e = effect;
    if e.object.is_set() {
        crate::input::effect_objects::release_effect(e);
    }
    fill_parameters(e);
    // The effect's function with a change of 1, the effect's number and 0.
    crate::input::effect_changes::effect_change_at(devices, e.change.get(), record, e, 1, n, 0)?;
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
