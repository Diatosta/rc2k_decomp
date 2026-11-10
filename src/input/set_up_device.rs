//! `input::set_up_device` (0x408BCD): a device's record made from a player's controls
//! (edi: `dword_75AE28` player 1, `dword_75AF94` player 2): +0x414 .. +0x4DB 0
//! (`system::zero_memory`, 0xC8 bytes); with a controller chosen ([edi] n, not 0): +0 n - 1,
//! +0x418 [edi+0x10], +0xC [edi+0x1C], +0x10 [edi+0x18], +0x14 [edi+0x14], +0x18 [edi+0xC],
//! +0x20 [edi+8] and +8 the same or 2 for 0, and the controller acquired (`input::acquire_device`);
//! then the twelve bindings: each its key (a byte of [edi+0x54 ..]) and the place, sign
//! and button of its word (`input::binding_place` of a dword of [edi+0x24 ..]), in the order
//! of `BINDINGS`.
//!
//! Takes: the device's record, the player's controls
//!
//! Caller: `input::device_command` (the race's start, both players). The acquiring through
//! `part` (0, esi and edi as here, eax [edi+8] or 2; in the game
//! `input::acquire_device`'s Rust version).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::types::InputDevice;
use crate::race::state_store::RaceCar;
use crate::session::Dword;
use std::mem::offset_of;

// The controls run from the car's `controller` (+0) to the key bytes (+0x54): the word 0x20
// is a gap no binding reads, 0x48 (`UNSET_BINDING`) the word the copies from the player's
// record pass over.
const _: () =
    assert!(offset_of!(RaceCar, bindings) + 4 * 11 - offset_of!(RaceCar, controller) == 0x50);
const _: () = assert!(offset_of!(RaceCar, keys) - offset_of!(RaceCar, controller) == 0x54);

/// The device `record` set up from the controls of `car`, from its `controller` on (see the
/// header).
pub fn set_up_device(record: &InputDevice, car: &RaceCar) -> Result<(), Refused> {
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
        // Converted: `rc2k_core::input::devices::device_kind`.
        let kind = word(8);
        record.sensitivity.set(kind);
        record
            .kind
            .set(rc2k_core::input::devices::device_kind(kind));
        crate::input::acquire_device::acquire_device(crate::input::owner::input(), record)?;
    }
    // Converted: `rc2k_core::input::devices::binding` over `DEVICE_BINDINGS`.
    for (binding, (key, controls_word, _)) in record
        .bindings
        .iter()
        .zip(rc2k_core::input::devices::DEVICE_BINDINGS)
    {
        let key = car.keys.get(key - 0x54);
        let b = rc2k_core::input::devices::binding(
            key,
            word(controls_word),
            record.acquired.get() == 1,
        );
        binding.key.set(b[0]);
        binding.place.set(b[1]);
        binding.sign.set(b[2]);
        binding.button.set(b[3]);
    }
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
