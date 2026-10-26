//! `sound::play_timed_slot` (0x40E59E): a sound slot of the race's sounds started when
//! asked, and marked done when its time is up.
//!
//! In:  ebp the slot, six dwords: +0 the sound, +4 1 = start it, +8 1 = playing, +0xC
//!      when it ends (`dword_5124E8` time), +0x10 the volume, +0x14 the rate (-0x6F: the
//!      sample's own); esi the request (`unk_5124BC`), its place already in it
//! Out: started: eax the end, ecx the volume, edx the rate (`sound::sample_settings`);
//!      checked: eax the clock; else nothing changed
//! +4 at 1 is cleared; then it starts unless it is playing (sound 0xE9 starts even
//! then): the
//! request's sound, volume (+8) and rate (+4, floats by `fild`) set, the end = the
//! clock + `sound::sample_duration`, +8 1, and `sound::play_placed_sound` (a tail jump).
//! Else, playing: +8 cleared once the clock is at or past the end (`js` on the
//! difference). 14 callers.
//! Refuses a null slot or request, and what the two leaves refuse, before writing.
//! A short jump in its first five bytes: private copy (0x73 bytes, the two calls and the
//! jump out relocated), manual patch.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose jump to `play_placed_sound` goes to a stand-in, against the
//! Rust version, on a slot of the test's, real samples.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::sample_settings;
use crate::sound::turn_parts::{CLOCK, PLACED, Part, turn_entry};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::slot_turn`.
pub fn play_timed_slot_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{SlotTurn, TimedSlot, from_int, slot_turn};
    let (slot, request) = (r.ebp as *mut [u32; 6], r.esi as *mut u32);
    if slot.is_null() || request.is_null() {
        return Err(BadInput("no slot or no request").into());
    }
    let w = unsafe { slot.read_unaligned() };
    let mut s = TimedSlot {
        sound: w[0],
        start: w[1],
        playing: w[2],
        ends: w[3],
        volume: w[4],
        rate: w[5],
    };
    let clock: u32 = unsafe { game_read(rva(CLOCK)) };
    let turn = slot_turn(&mut s, clock, sample_settings, sample_duration)?;
    unsafe { slot.write_unaligned([s.sound, s.start, s.playing, s.ends, s.volume, s.rate]) };
    match turn {
        SlotTurn::Nothing => Ok(()),
        SlotTurn::Checked => {
            r.eax = clock;
            Ok(())
        }
        SlotTurn::Start { volume, rate, ends } => {
            unsafe {
                request.write_unaligned(s.sound);
                request.add(2).write_unaligned(from_int(volume as i32));
                request.add(1).write_unaligned(from_int(rate as i32));
            }
            (r.eax, r.ecx, r.edx) = (ends, volume, rate);
            part(PLACED, r)
        }
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, play_timed_slot_in)
}
