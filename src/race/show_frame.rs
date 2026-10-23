//! `race::show_frame` (0x4547AE): the race frame's end, after its 3D scene: the frame
//! locked (`scene::lock_frame`; carry: lost), the race's sound tick (eax 0x13), the credits
//! rolled while `dword_71C3E0` is set (`race::roll_credits`, eax 1), the frame rate drawn
//! (`race::draw_frame_rate`), the frame unlocked and shown (`scene::unlock_frame`,
//! `scene::present_frame`; carry: lost); the sound tick (eax 0x14), the clock restarted
//! (`scene::restart_clock`) and its time less `dword_63DC38` into `dword_63DC3C`, the frame
//! rate measured (`race::measure_frame_rate`); carry clear.
//!
//! In:  nothing
//! Out: carry set when the frame was lost (the lock or the showing failed), else clear; the
//!      registers as the last call leaves them
//!
//! Caller: `drive::race_frame` (its part 7). It begins with a call: a private copy (its
//! nine calls relocated) and a manual patch. Its calls through `part` by place
//! (`CALLEES`); in the game the sound tick's and the measuring's Rust versions, the others
//! (all hooked) through the game. Shadow mode runs the original alone (it shows the
//! frame). Self-test: a copy whose calls go to stand-ins (the lock and the showing failing
//! or not), against the Rust version, with and without the credits.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 8] = [
    0x44E837, // 0 scene::lock_frame
    0x421BA9, // 1 sound::race_sound_tick
    0x415FC2, // 2 race::roll_credits
    0x4562B2, // 3 race::draw_frame_rate
    0x44E851, // 4 scene::unlock_frame
    0x44E895, // 5 scene::present_frame
    0x450381, // 6 scene::restart_clock
    0x42009E, // 7 race::measure_frame_rate
];

const CREDITS_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const CLOCK_BASE_RVA: usize = rva(0x63DC38); // dword_63DC38
const FRAME_TIME_RVA: usize = rva(0x63DC3C); // dword_63DC3C

/// A call by place (`CALLEES`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers and carry it leaves.
pub unsafe fn show_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(0, &mut r)?;
    if r.carry() {
        return Ok(r);
    }
    r.eax = 0x13;
    part(1, &mut r)?;
    if unsafe { game_read::<u32>(CREDITS_RVA) } != 0 {
        r.eax = 1;
        part(2, &mut r)?;
    }
    for place in [3, 4, 5] {
        part(place, &mut r)?;
    }
    if r.carry() {
        return Ok(r);
    }
    r.eax = 0x14;
    part(1, &mut r)?;
    part(6, &mut r)?;
    r.eax = r.eax.wrapping_sub(unsafe { game_read(CLOCK_BASE_RVA) });
    unsafe { game_write(FRAME_TIME_RVA, r.eax) };
    part(7, &mut r)?;
    r.set_carry(false);
    Ok(r)
}

/// In the game: the sound tick's and the measuring's Rust versions, the others' through
/// the game (their hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => crate::sound::race_sound_tick::race_sound_tick_in(
            r,
            &mut crate::sound::turn_parts::game_part,
        ),
        7 => {
            let flags = r.eflags;
            *r = Regs {
                eflags: flags,
                ..unsafe {
                    // Converted: the measuring's `_plain` version (the frame's comparison
                    // runs the original alone).
                    crate::race::measure_frame_rate::measure_frame_rate_plain(
                        r,
                        &mut crate::race::measure_frame_rate::rdtsc,
                    )
                }
            };
            Ok(())
        }
        _ => {
            crate::calls::call(rva(CALLEES[place as usize]), r);
            Ok(())
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { show_frame(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
