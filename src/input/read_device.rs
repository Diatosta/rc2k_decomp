//! A device's four readings (steering, brake, throttle, the bits) in the race, from the
//! controller as SDL reads it through the player's profile for it (`input::profiles`,
//! `rc2k_core::input::profile::readings`) and the keys the player bound: a key or a digital
//! binding gives all of a reading, setting bit 0x1000 (steering), 0x4000 (throttle) or 0x2000
//! (brake); an axis binding gives its value. Eight buttons become bits 1 .. 0x80 and the hat
//! the top two bytes of the bits. A device not being read keeps its readings, one that is not
//! there reads as its keys.

use crate::errors::Refused;
use crate::input::devices::key_table;
use crate::input::owner::Input;
use crate::input::types::InputDevice;
use rc2k_core::input::profile::{ACTIONS, Keys};

/// The device `record` (number `device`) read (see the header). The poll that copies the
/// device's state into the record, for the pages that show it, comes first.
pub(super) fn read_device(
    input: &Input,
    device: usize,
    record: &InputDevice,
) -> Result<(), Refused> {
    let devices = &input.devices;
    crate::input::poll_device::poll_device(devices, record)?;
    if record.reading.get() != 1 {
        return Ok(());
    }
    // None for a player on the keyboard: the keys alone.
    let profile = crate::input::profiles::race_profile(input, device);
    let table = key_table(devices);
    let mut keys = Keys::default();
    for (down, binding) in keys.0.iter_mut().zip(record.bindings.iter()) {
        *down = table[binding.key.get() as usize & 0xFF] & 0x80 != 0;
    }
    debug_assert_eq!(record.bindings.len(), ACTIONS);
    let raw = crate::input::di_objects::profile_reading(record.device.get());
    // The controller's profile says whether its hat and buttons are a shifter's.
    let shifter = profile.as_ref().is_some_and(|p| p.shifter);
    let r = rc2k_core::input::profile::readings(profile.as_ref(), raw.as_ref(), &keys, shifter);
    // With the H shifter, a gate asks for its gear and the lever out of the gates leaves the gear
    // up / down bindings working (each controller's gate state kept between readings).
    let bits = if shifter {
        static GATES: std::sync::Mutex<Vec<rc2k_core::input::profile::ShifterGate>> =
            std::sync::Mutex::new(Vec::new());
        let mut gates = GATES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gates.len() <= device {
            gates.resize(device + 1, Default::default());
        }
        gates[device].next(r.bits)
    } else {
        r.bits
    };
    record.readings[0].set(r.steering);
    record.readings[1].set(r.brake);
    record.readings[2].set(r.throttle);
    record.bits.set(bits);
    Ok(())
}
