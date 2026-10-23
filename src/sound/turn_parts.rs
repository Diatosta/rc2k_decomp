//! The race sounds' turn (stream B, 2026-10-18): `sound::race_turn` (0x40D5F3) and the
//! trees under it, which work on the eight cars' sound objects (`g_raceSoundObjects`,
//! 0x4A8 bytes each), the shared spots' voices (`unk_5168C0`) and one request
//! (`unk_5124BC`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: their calls by place, the place being the callee's IDA address: in the game
//! the Rust versions, in the tests stand-ins that note each call with its registers (and
//! the request when esi points at it) and change nothing but eax for the three that give
//! a number; and copies of the originals whose calls (and jumps out) go to them.
//!
//! Shadow mode runs the originals that send sounds alone (a sound must not be sent
//! twice); those that only work out state run both, compared (`dispatch_turn`).

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::service::harness::Region;

/// A Rust version: the registers as the original leaves them.
pub type Rust = fn(&mut Regs) -> Result<(), Refused>;

/// unk_5124BC: the request the turn's sounds go with.
pub const REQUEST: usize = 0x5124BC;
pub const REQUEST_LEN: usize = 0x28;
/// dword_5124E8: the time of the sounds' last turn (`sound/race_clock.rs`).
pub const CLOCK: usize = 0x5124E8;
/// g_raceSoundObjects: eight pointers.
pub const OBJECTS: usize = 0x512A10;
/// dword_514F7C: the sound objects in use.
pub const COUNT: usize = 0x514F7C;
/// dword_5183CC: the scratch dword the x87 code goes through.
pub const SCRATCH: usize = 0x5183CC;

/// `sound::play_sound` with the request at esi.
pub fn play(r: &Regs) -> Result<(), Refused> {
    crate::sound::play_sound::play_sound_in_game(r);
    Ok(())
}

/// `sound::stop_sound`: the request at esi stopped (and its slot's sound played again
/// when it says so).
pub fn stop(r: &Regs) -> Result<(), Refused> {
    if let Some(with) = unsafe { crate::sound::stop_sound::stop_sound(r) }? {
        crate::sound::play_sound::play_sound_in_game(&with);
    }
    Ok(())
}

/// `sound::play_placed_sound` with the request at esi.
pub fn placed(r: &Regs) -> Result<(), Refused> {
    unsafe {
        let mode = game_read(crate::sound::play_placed_sound::MODE_RVA);
        let samples = game_addr(crate::sound::play_placed_sound::SAMPLES_RVA);
        crate::sound::play_placed_sound::play_placed_sound(r, mode, samples, &mut |w| {
            crate::sound::play_sound::play_sound_in_game(w)
        })?;
    }
    Ok(())
}

/// `sound::step_race_clock`: true when a turn is due (the original's carry clear).
pub fn step_clock() -> bool {
    unsafe { crate::sound::race_clock::step_race_clock(crate::system::tick_count::tick_count()) }
}

// ---- the dispatch ----

/// `dispatch_turn` / `dispatch_turn_floats` without the hook (no shadow mode; a refusal is
/// the caller's): the Rust version with the game's parts, its registers but esp and the
/// flags given back.
pub fn turn_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    let mut g = *r;
    rust(&mut g)?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}

// ---- the tests' side ----

/// The request and the scratch dword, which every one of these may write.
pub fn common_regions() -> Vec<Region> {
    vec![
        (game_addr(rva(REQUEST)) as u32, REQUEST_LEN),
        (game_addr(rva(SCRATCH)) as u32, 4),
    ]
}
