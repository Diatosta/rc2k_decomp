//! The six force-feedback effects' change functions (`effect_0_change` .. `effect_5_change`),
//! which `input::device_effects` calls with a change waiting: the change `what` (1 set up, 2
//! start, 3 values, 4 stop), `a` and `b` its two signed values, the device's record and the
//! effect (0xA4 bytes). Each effect type sets its parameters (a value v is
//! `(v x 10000, as a signed dword) >> 8`) and flags as the code below says; other changes do
//! nothing. The DirectInput calls go through `effect_objects`. Refuses (before anything) for
//! effect 1 a row past the table's 14 and a blend of 0 (it divides by it).

use crate::errors::{BadInput, Refused};
use crate::input::store::Devices;
use crate::input::types::{ForceEffect, InputDevice};

// `input::start_effect`
// `input::set_effect_parameters`
// `input::stop_effect`

/// The change `what` (with `a`, `b`) of the effect `effect` of the device `record`, by
/// the effect's function `number` (`input::effect_N_change`).
pub fn effect_change(
    devices: &Devices,
    number: u32,
    record: &InputDevice,
    effect: &ForceEffect,
    what: u32,
    a: u32,
    b: u32,
) -> Result<(), Refused> {
    use crate::input::effect_objects::{set_effect_parameters, start_effect, stop_effect};
    use rc2k_core::input::effect_changes::{Call, Field, plan};
    let e = effect;
    let p = plan(
        number,
        what,
        a,
        b,
        || record.sensitivity.get(),
        // the core asks only for a row inside the table
        |n| {
            devices.effect_rows[n as usize]
                .each_ref()
                .map(crate::session::Dword::get)
        },
    )
    .map_err(BadInput::from)?;
    for (f, v) in p.stores {
        match f {
            Field::Duration => e.effect.duration.set(v),
            Field::FirstAxis => e.axes[0].set(v),
            // the `DIEFFECT` points at the effect's own envelope
            Field::EnvelopePointer => e.effect.envelope.set(crate::input::make_effect::PART_SET),
            Field::FadeTime => e.envelope.fade_time.set(v),
            Field::Specific(n) => e
                .type_specific
                .get(n as usize)
                .ok_or(BadInput("a type-specific parameter past the six"))?
                .set(v),
            Field::Found => e.found.set(v),
        }
    }
    match p.call {
        Some(Call::Start) => start_effect(e),
        Some(Call::Set(flags)) => set_effect_parameters(e, flags),
        Some(Call::Stop) => stop_effect(e),
        None => {}
    }
    Ok(())
}

/// The six functions' addresses (`input::effect_0_change` .. `_5_`), in number order.
const FUNCTIONS: [u32; 6] = [0x409708, 0x4097FA, 0x409754, 0x4097B4, 0x409871, 0x4098BF];

/// The effect's function (`ForceEffect::change`, the game's address of one of the six)
/// with the change.
pub fn effect_change_at(
    devices: &Devices,
    function: u32,
    record: &InputDevice,
    effect: &ForceEffect,
    what: u32,
    a: u32,
    b: u32,
) -> Result<(), Refused> {
    let Some(number) = FUNCTIONS.iter().position(|&f| f == function) else {
        return Err(BadInput("an effect's function that is none of the six").into());
    };
    effect_change(devices, number as u32, record, effect, what, a, b)
}
