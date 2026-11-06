//! The six force-feedback effects' functions (+0 of each effect of an input device's
//! record: `input::effect_0_change` 0x409708 for the first effect, `effect_1_change`
//! 0x4097FA, `effect_2_change` 0x409754, `effect_3_change` 0x4097B4, `effect_4_change`
//! 0x409871, `effect_5_change` 0x4098BF; both records hold the same six), which
//! `input::device_effects` calls with a change waiting: eax the change (1 set up, 2 start,
//! 3 values, 4 stop), ecx and edx its two values (signed), esi the device's record, edi the
//! effect (0xA4 bytes; +0x30 its DirectInput effect, +0x34 its parameters).
//! A value v below is `(v x 10000, as a signed dword) >> 8` (arithmetic).
//! - effect 0: 2 start; 3 +0x6C and +0x70 ecx's, +0x68 edx's, the parameters set with
//!   flags 0x100; 4 stop.
//! - effect 1: 1 +0x68 0; 2 start; 3 by the row ecx of `unk_4F6E08` (four dwords each): +0x68
//!   the row's first two blended by edx (`(b - a) x edx >> 8 + a`, then as a value), +0x74
//!   10000000 / the next two blended likewise (`idiv`); flags 0x100; 4 stop.
//! - effect 2: 1 +0x3C 500000, +0x5C the effect's +0x90, +0xA0 500000, +0x74 100000, +0x68
//!   0; 3 +0x68 ecx's, flags 0x20000100; 4 stop.
//! - effect 3: 1 +0x3C 250000, +0x68 0; 3 +0x68 ecx's, flags 0x20000100; 4 stop.
//! - effect 4: 1 +0x6C and +0x70 0; 2 start; 3 +0x6C and +0x70 ecx's, flags 0x100; 4 stop.
//! - effect 5: 1 +0x3C 200000, +0x80 4, +0x5C the effect's +0x90, +0xA0 200000, +0x68 0, and
//!   with the record's +0x20 above 1 (unsigned) +0xC 0; 3 +0x68 ecx's, flags 0x20000100; 4
//!   stop.
//!
//! Other changes do nothing.
//!
//! In:  as above
//! Out: every register kept (`pusha` .. `popa`)
//! The DirectInput calls (`sub_409AEE` the effect's Download and Start, `sub_409B1F` its
//! SetParameters with eax the flags, `sub_409B0D` its Stop; each only with an effect at
//! +0x30) are the Rust versions (`effect_objects`). Refuses
//! (before anything) for effect 1 a row past the table's 14 and a blend of 0 (the
//! original's `idiv` traps).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::input::types::{ForceEffect, InputDevice};

// sub_409AEE
// sub_409B1F
// sub_409B0D

/// The change `what` (with `a`, `b`) of the effect `effect` of the device `record`, by
/// the effect's function `number` (`input::effect_N_change`).
pub fn effect_change(
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
    // Converted: `rc2k_core::input::effect_changes::plan`.
    let p = plan(
        number,
        what,
        a,
        b,
        || record.sensitivity.get(),
        // the core asks only for a row inside the table
        |n| {
            crate::input::owner::input().devices.effect_rows[n as usize]
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
    effect_change(number as u32, record, effect, what, a, b)
}
