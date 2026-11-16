//! A device's record made from a player's controls
//! (player 1's or player 2's settings): +0x414 .. +0x4DB 0
//! (`system::zero_memory`, 0xC8 bytes); with a controller chosen (the car's `controller` n, not 0): +0 n - 1,
//! +0x418 `controller_mode`, +0xC `strength`, +0x10 `centre_dead_zone_pct`, +0x14
//! `end_zone_pct`, +0x18 `axis_codes`, +0x20 `sensitivity_level` and +8 the same or 2 for 0, and the controller acquired (`input::acquire_device`);
//! then the twelve bindings' keys (bytes of `keys`), in the order of `BINDINGS`; and the
//! profile the device reads through: the profile of the player whose controls the car took
//! (`Input::race_players`) for the controller (`input::profiles`), none for the keyboard.

use crate::errors::Refused;
use crate::input::types::InputDevice;
use crate::memory::session::Dword;
use crate::race::state_store::PlayerControls;
#[cfg(test)]
use std::mem::offset_of;

// The controls run from the car's `controller` (+0) to the key bytes (+0x54): the word 0x20
// is a gap no binding reads, 0x48 (`UNSET_BINDING`) the word the copies from the player's
// record pass over.
#[cfg(test)]
const _: () = assert!(
    offset_of!(PlayerControls, bindings) + 4 * 11 - offset_of!(PlayerControls, controller) == 0x50
);
#[cfg(test)]
const _: () =
    assert!(offset_of!(PlayerControls, keys) - offset_of!(PlayerControls, controller) == 0x54);

/// The device `record` set up from the controls of `car`, from its `controller` on (see the
/// header).
pub fn set_up_device(
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
    device: usize,
    record: &InputDevice,
    car: &PlayerControls,
) -> Result<(), Refused> {
    // The controls' words by their offset from `controller` (a quarter of it the index).
    let words: [&Dword; 21] = [
        &car.controller,
        &car.gearing,
        &car.sensitivity_level,
        &car.axis_codes,
        &car.controller_mode,
        &car.end_zone_pct,
        &car.centre_dead_zone_pct,
        &car.strength,
        &car._unused_4c,
        &car.bindings[0],
        &car.bindings[1],
        &car.bindings[2],
        &car.bindings[3],
        &car.bindings[4],
        &car.bindings[5],
        &car.bindings[6],
        &car.bindings[7],
        &car.bindings[8],
        &car.bindings[9],
        &car.bindings[10],
        &car.bindings[11],
    ];
    // +0x414 .. +0x4DC: the reading flag, the hat's kind and the bindings.
    record.reading.set(0);
    record.hat_kind.set(0);
    for binding in &record.bindings {
        binding.key.set(0);
        binding.place.set(0);
        binding.sign.set(0);
        binding.button.set(0);
    }
    let word = |from: usize| words[from / 4].get();
    let n = word(0);
    if n != 0 {
        record.controller.set(n - 1);
        for (to, from) in [
            (&record.hat_kind, 0x10),
            (&record.strength, 0x1C),
            (&record.dead_zone, 0x18),
            (&record.end_zone, 0x14),
            (&record.axis_code, 0xC),
        ] {
            to.set(word(from));
        }
        let kind = word(8);
        record.sensitivity.set(kind);
        record
            .kind
            .set(rc2k_core::input::devices::device_kind(kind));
        crate::input::acquire_device::acquire_device(input, system, record)?;
    }
    for (binding, (key, _, _)) in record
        .bindings
        .iter()
        .zip(rc2k_core::input::devices::DEVICE_BINDINGS)
    {
        binding.key.set(car.keys.get(key - 0x54) as u32);
    }
    let profile = (n != 0 && record.acquired.get() == 1)
        .then(|| {
            let player = crate::input::profiles::race_player(input, device);
            crate::input::profiles::with_profile(input, player, n, |p| p.clone())
        })
        .flatten();
    crate::input::profiles::set_race_profile(input, device, profile);
    Ok(())
}

#[cfg(test)]
mod tests {
    /// The bindings fill the places the core lists, a binding after another.
    #[test]
    fn the_bindings_stand_where_the_core_puts_them() {
        for (n, (_, _, to)) in rc2k_core::input::devices::DEVICE_BINDINGS
            .into_iter()
            .enumerate()
        {
            assert_eq!(to, 0x41C + 16 * n);
        }
    }
}
