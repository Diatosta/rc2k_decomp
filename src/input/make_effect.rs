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
//! Caller: `input::begin_force` (each of the six). The effect's function through `part`
//! (its address; eax 1, the other registers as here; in the game through the game, one of
//! `input::effect_N_change`), the methods the objects' own. Shadow mode runs the original
//! alone (it calls DirectInput). Self-test: the original and the Rust version on
//! DirectInput objects made in Rust (`input::di_harness`), the effect's function a noting
//! stand-in: each kind and one past them, found or not, an object before or not, the wheel
//! (force feedback) and the joystick (none: `CreateEffect` fails).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::input::devices::{at, put};
use crate::input::di_harness::{method, slot};
use crate::input::types::{ForceEffect as F, InputDevice};

/// The effect's `DIEFFECT` and parameters, as the original writes them (in its order).
pub fn fill_parameters(e: u32) {
    // Converted: `rc2k_core::input::force` (the figures, `type_specific`).
    use rc2k_core::input::force as k;
    let w = |offset: u32, v: u32| put(e, offset, v);
    w(field!(F, effect.size), k::EFFECT_SIZE);
    w(field!(F, effect.flags), k::EFFECT_FLAGS);
    w(field!(F, effect.duration), k::FOR_EVER);
    w(field!(F, effect.sample_period), 0);
    w(field!(F, effect.gain), k::FULL_GAIN);
    w(field!(F, effect.trigger_button), k::NO_BUTTON);
    w(field!(F, effect.trigger_repeat_interval), 0);
    w(field!(F, effect.axes_count), k::AXES_COUNT);
    w(field!(F, effect.axes), e.wrapping_add(field!(F, axes)));
    w(field!(F, axes), k::AXES[0]);
    w(field!(F, axes) + 4, k::AXES[1]);
    w(
        field!(F, effect.directions),
        e.wrapping_add(field!(F, directions)),
    );
    w(field!(F, directions), k::DIRECTION);
    w(field!(F, effect.envelope), 0);
    w(field!(F, effect.type_specific_size), 0);
    w(field!(F, effect.type_specific), 0);
    // an envelope of its size, all 0, not linked
    w(field!(F, envelope.size), k::ENVELOPE_SIZE);
    for offset in [
        field!(F, envelope.attack_level),
        field!(F, envelope.attack_time),
        field!(F, envelope.fade_level),
        field!(F, envelope.fade_time),
    ] {
        w(offset, 0);
    }
    let params = k::type_specific(at(e, field!(F, kind)));
    if params.is_empty() {
        return;
    }
    w(
        field!(F, effect.type_specific_size),
        4 * params.len() as u32,
    );
    w(
        field!(F, effect.type_specific),
        e.wrapping_add(field!(F, type_specific)),
    );
    for (n, &v) in params.iter().enumerate() {
        w(field!(F, type_specific) + 4 * n as u32, v);
    }
}

/// The Rust version (see the header).
pub fn make_effect(entry: &Regs) -> Result<(), Refused> {
    let e = entry.edi;
    if at(e, field!(F, object)) != 0 {
        crate::input::effect_objects::release_effect(e);
    }
    fill_parameters(e);
    let mut r = Regs { eax: 1, ..*entry };
    crate::input::make_effect::game_part(at(e, field!(F, change)) as usize, &mut r)?;
    if at(e, field!(F, found)) == 1 {
        method(
            at(entry.esi, field!(InputDevice, device)),
            slot::CREATE_EFFECT,
            &[
                e.wrapping_add(field!(F, guid)),
                e.wrapping_add(field!(F, effect)),
                e.wrapping_add(field!(F, object)),
                0,
            ],
        );
    }
    Ok(())
}

/// In the game: the effect's function through the game.
pub fn game_part(function: usize, r: &mut Regs) -> Result<(), Refused> {
    crate::menu::table_calls::call_any(function as u32, r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    make_effect(r)?;
    Ok(())
}
