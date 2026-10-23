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
//! +0x30) go through `part` (0, 1, 2), in the game their Rust versions (`effect_objects`). Refuses
//! (before anything) for effect 1 a row past the table's 14 and a blend of 0 (the
//! original's `idiv` traps). Shadow mode runs the original alone (DirectInput). Manual
//! patches with private copies (a `jnz` in the first bytes). Self-test: a copy whose calls
//! go to stand-ins, against the Rust version, each change, values of both signs.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::{Regs, game_addr, rva};
use crate::input::types::{ForceEffect, InputDevice};
use crate::menu::parts::Part;

// sub_409AEE
// sub_409B1F
// sub_409B0D
const ROWS_RVA: usize = rva(0x4F6E08); // unk_4F6E08

fn put(effect: u32, o: u32, v: u32) {
    unsafe { ((effect + o) as *mut u32).write_unaligned(v) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version: the registers it leaves.
pub unsafe fn effect_change(entry: &Regs, number: u32, part: Part) -> Result<Regs, Refused> {
    use rc2k_core::input::effect_changes::{Call, Field, plan};
    let mut r = *entry;
    let e = entry.edi;
    let row_at = |n: u32| game_addr(ROWS_RVA) as u32 + 16 * n;
    // Converted: `rc2k_core::input::effect_changes::plan`.
    let p = plan(
        number,
        entry.eax,
        entry.ecx,
        entry.edx,
        || get(entry.esi + field!(InputDevice, sensitivity)),
        |n| std::array::from_fn(|k| get(row_at(n) + 4 * k as u32)),
    )
    .map_err(BadInput::from)?;
    for (f, v) in p.stores {
        let (at, v) = match f {
            Field::Duration => (field!(ForceEffect, effect.duration), v),
            Field::FirstAxis => (field!(ForceEffect, axes), v),
            Field::EnvelopePointer => (
                field!(ForceEffect, effect.envelope),
                e + field!(ForceEffect, envelope),
            ),
            Field::FadeTime => (field!(ForceEffect, envelope.fade_time), v),
            Field::Specific(n) => (field!(ForceEffect, type_specific) + 4 * n, v),
            Field::Found => (field!(ForceEffect, found), v),
        };
        put(e, at, v);
    }
    if let Some((divisor, remainder)) = p.division {
        // The registers at the call: the row, the divisor and the `idiv`'s remainder.
        r.ebx = row_at(entry.ecx);
        r.ecx = divisor;
        r.edx = remainder;
    }
    match p.call {
        Some(Call::Start) => part(0, &mut r)?,
        Some(Call::Set(flags)) => {
            r.eax = flags;
            part(1, &mut r)?;
        }
        Some(Call::Stop) => part(2, &mut r)?,
        None => {}
    }
    Ok(*entry)
}

/// In the game: the Rust versions of the effect's calls (`input::effect_objects`).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::input::effect_objects::{set_effect_parameters, start_effect, stop_effect};
    match place {
        0 => start_effect(r.edi),
        1 => set_effect_parameters(r.edi, r.eax),
        _ => stop_effect(r.edi),
    }
    Ok(())
}

/// `dispatch_effect` without its hook (a refusal is the caller's).
pub fn effect_entry(r: &mut Regs, number: u32) -> Result<(), Refused> {
    crate::menu::parts::parted_entry(
        r,
        &|r, part| unsafe { effect_change(r, number, part) },
        game_part,
    )
}
