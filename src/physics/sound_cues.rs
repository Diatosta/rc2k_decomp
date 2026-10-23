//! The car's sound cues (IDA 2026-10-18): `physics::off_line_warnings` (0x44148F) counts
//! the car off its line and turned from it and, now and then, cues a sound by
//! `physics::cue_listed_car_sound` (0x45D1BC) -> `physics::cue_car_sound` (0x40ECE7) ->
//! `physics::queue_sound_cue` (0x40EE5C), which may stop the playing one by
//! `sound::play_sound`; the pace notes by `physics::cue_listed_car_sound_last` (0x45D1D3)
//! -> `physics::cue_car_sound_last` (0x40ECBB) -> the same queue. This file: their calls by
//! place (0 `play_sound`, 1 `queue_sound_cue`, 2 `cue_car_sound`, 3
//! `cue_listed_car_sound`, 4 `cue_car_sound_last`, 5 `cue_listed_car_sound_last`): in the game the Rust
//! versions, in the tests stand-ins that note the calls and their registers (and change
//! none), and copies of the originals whose calls go to them.
//!
//! Shadow mode runs these originals alone (a sound command must not be sent twice).

use crate::errors::Refused;
use crate::framework::Regs;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub const PLAY: u32 = 0;
pub const QUEUE: u32 = 1;
pub const CUE: u32 = 2;
pub const LISTED: u32 = 3;
/// The pace notes' pair (`physics::cue_car_sound_last`, `physics::cue_listed_car_sound_last`).
pub const CUE_LAST: u32 = 4;
/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        PLAY => crate::sound::play_sound::play_sound_in_game(r),
        QUEUE => crate::physics::queue_sound_cue::queue_sound_cue_in(r, &mut game_part)?,
        CUE => crate::physics::cue_car_sound::cue_car_sound_in(r, &mut game_part)?,
        LISTED => crate::physics::cue_listed_car_sound::cue_listed_car_sound_in(r, &mut game_part)?,
        CUE_LAST => crate::physics::cue_car_sound_last::cue_car_sound_last_in(r, &mut game_part)?,
        _ => crate::physics::cue_listed_car_sound_last::cue_listed_car_sound_last_in(
            r,
            &mut game_part,
        )?,
    }
    Ok(())
}

/// `dispatch_cue` without its hook (Rust callers; a refusal is the caller's).
pub fn cue_entry(
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
