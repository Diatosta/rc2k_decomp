//! A request for one of a device's six force-feedback effects: with the record's effects on
//! (+0x28 1), and for a "set" (3) only while the race runs, `kind | (a & 0xFFF) << 8 | (b &
//! 0xFFF) << 20` into the effect's +0x2C (effect n at +0x3C + n * 0xA4), where the input thread
//! takes it (`input::device_effects`). See `input/effects.rs`. Refuses (before anything) an
//! effect past 5.

use crate::errors::{BadInput, Refused};
use crate::input::store::ForceFeedback;
use crate::input::types::InputDevice;

/// Effect `effect` (0 .. 5) of the device `record` told `what` (2 start, 3 set, 4
/// stop) with the numbers `a` and `b`; "set" only while the race runs (`race_on`).
pub fn set_effect(
    feedback: &ForceFeedback,
    record: &InputDevice,
    effect: u32,
    what: u32,
    a: u32,
    b: u32,
) -> Result<(), Refused> {
    use rc2k_core::input::effects::{request, wanted};
    let race_on: u32 = feedback.race_on.get();
    if wanted(record.effects_on.get() == 1, race_on == 1, effect, what)? {
        record
            .effects
            .get(effect as usize)
            .ok_or(BadInput("an effect past the six"))?
            .pending
            .set(request(what, a, b));
    }
    Ok(())
}
