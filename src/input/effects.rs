//! The controllers' force feedback, the game's side (read 2026-10-12, ported 2026-10-19):
//! what the race asks of the six effects of each device record (+0x3C, 0xA4 bytes each).
//! A request is a dword at the effect's +0x2C (`input::set_effect`: what to do in the low
//! byte, 2 start, 3 set, 4 stop; two signed 12-bit numbers above it), which the input
//! thread hands to DirectInput (`input::device_effects`, the platform's side). Nothing
//! here calls DirectInput.
//!
//! The calls by place (the callee's IDA address): in the game the Rust versions, in the
//! tests the noting stand-ins of `sound::turn_parts`.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub const SET_EFFECT: u32 = 0x40961F; // input::set_effect
pub const START_DEVICE: u32 = 0x409672; // input::start_device_effects
pub const STOP_DEVICE: u32 = 0x40969E; // input::stop_device_effects
pub const START: u32 = 0x409659; // input::start_effects
pub const STOP: u32 = 0x409685; // input::stop_effects
pub const CAR: u32 = 0x40F347; // input::update_car_effects
pub const TICK: u32 = 0x411024; // system::tick_count

/// dword_518090: 1 while the race runs (`input::device_command` 3 .. 2); "set" requests
/// (3) are dropped while it is not.
pub const RACE_ON_RVA: usize = rva(0x518090);
/// g_inputRecord1, g_inputRecord2: the two device records.
pub const RECORD_1: usize = 0x4F6EF0;
pub const RECORD_2: usize = 0x4F73E8;
/// g_effectsLastTime: the clock at the effects' last step.
pub const LAST_TIME: usize = 0x518084;
/// The record's +0x28: 1 while it has effects.
pub const HAS_EFFECTS: u32 = 0x28;
pub const EFFECTS: u32 = 0x3C;
pub const EFFECT_LEN: u32 = 0xA4;
pub const REQUEST: u32 = 0x2C;

/// The calls in the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    let on = unsafe { game_read::<u32>(RACE_ON_RVA) };
    match place {
        SET_EFFECT => crate::input::set_effect::set_effect(r, on)?,
        START_DEVICE => {
            crate::input::start_device_effects::start_device_effects(r, &mut game_part)?
        }
        STOP_DEVICE => crate::input::stop_device_effects::stop_device_effects(r, &mut game_part)?,
        START => crate::input::start_effects::start_effects(r, &mut game_part)?,
        STOP => crate::input::stop_effects::stop_effects(r, &mut game_part)?,
        CAR => crate::input::update_car_effects::update_car_effects(r, &mut game_part)?,
        TICK => r.eax = crate::system::tick_count::tick_count(),
        _ => return Err(BadInput("a force feedback call with no version").into()),
    }
    Ok(())
}

/// `dispatch_effects` without its hook (no shadow mode; a refusal is the caller's).
pub fn effects_entry(
    r: &mut Regs,
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let mut g = *r;
    rust(&mut g, &mut game_part)?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}
