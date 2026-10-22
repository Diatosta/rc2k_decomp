//! `drive::race_frame` (0x4541BF): one frame of the race. The window's messages
//! (`system::pump_messages`); quitting (`g_quitting` 1) ends it there. With the window
//! the active one and `dword_518380` not 1: the 3D scene begun (`scene::begin_scene`;
//! its carry: the frame lost), drawn (`race::draw_race_scene`) and ended (`scene::end_scene`), the
//! race's sounds (`sound::race_sound_tick`, eax 0x12), the replay's screenshot
//! (`race::replay_screenshot`), the frame shown (`race::show_frame`, carry: lost); made: carry clear. Lost (or
//! not drawn): unless quitting, the surfaces restored (`sub_405B10`, ebx `dword_518380`)
//! and the flags of `cmp (ebx ^ 1) | dword_518380, 1`.
//!
//! In:  nothing
//! Out: carry clear when the frame was made; otherwise the flags above (quitting: those
//!      of `cmp g_quitting, 1`); the registers as the calls leave them, ebx on the lost way
//!
//! Callers: `drive::race_loop`, `quit_question`, `pause_until_key`, `finish_run`,
//! `sub_4607A5`. The drawing, the sound tick, the screenshot and the showing are their Rust
//! versions; the window, the scene's begin and end and the restore through the game. A private
//! copy (a call at +0).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const ACTIVE_RVA: usize = rva(0x518374); // g_windowActive
const LOST_RVA: usize = rva(0x518380); // dword_518380

/// The calls by place, as IDA shows them.
const CALLED: [usize; 9] = [
    0, 0x410535, // 1 system::pump_messages
    0x44E8A1, // 2 scene::begin_scene
    0x45429B, // 3 race::draw_race_scene
    0x44E952, // 4 scene::end_scene
    0x421BA9, // 5 sound::race_sound_tick
    0x41F3DB, // 6 race::replay_screenshot
    0x4547AE, // 7 race::show_frame
    0x405B10, // 8 sub_405B10: the surfaces restored
];

/// A call by place (`CALLED`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// The Rust version (see the header): the registers and flags it leaves.
pub fn race_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let cmp_one = |r: &mut Regs, v: u32| {
        r.set_carry(v < 1);
        r.set_zero(v == 1);
    };
    part(1, &mut r)?;
    if g(QUITTING_RVA) == 1 {
        // The game is closing. In the game its shutdown thread (`threading::request_quit`)
        // ends the program meanwhile; the standalone, which has none, ends it here.
        #[cfg(feature = "standalone")]
        crate::standalone::leave_from_race();
        #[cfg(not(feature = "standalone"))]
        {
            cmp_one(&mut r, 1);
            return Ok(r);
        }
    }
    'lost: {
        if g(ACTIVE_RVA) == 0 || g(LOST_RVA) == 1 {
            break 'lost;
        }
        part(2, &mut r)?;
        if r.carry() {
            break 'lost;
        }
        part(3, &mut r)?;
        part(4, &mut r)?;
        r.eax = 0x12;
        part(5, &mut r)?;
        part(6, &mut r)?;
        part(7, &mut r)?;
        if r.carry() {
            break 'lost;
        }
        r.set_carry(false);
        return Ok(r);
    }
    if g(QUITTING_RVA) == 1 {
        cmp_one(&mut r, 1);
        return Ok(r);
    }
    r.ebx = g(LOST_RVA);
    part(8, &mut r)?;
    r.ebx = (r.ebx ^ 1) | g(LOST_RVA);
    let v = r.ebx;
    cmp_one(&mut r, v);
    Ok(r)
}

/// In the game: the Rust versions of the drawing, the sound tick, the replay screenshot and
/// the showing; the rest (the messages, the scene's begin and end, the surfaces' restore)
/// through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 5 {
        use crate::sound::race_sound_tick::race_sound_tick_in;
        return race_sound_tick_in(r, &mut crate::sound::turn_parts::game_part);
    }
    if place == 3 {
        use crate::race::draw_race_scene::{draw_race_scene, game_part};
        *r = unsafe { draw_race_scene(r, &mut game_part) }?;
        return Ok(());
    }
    if place == 6 {
        use crate::race::replay_screenshot::{game_part, replay_screenshot};
        *r = unsafe { replay_screenshot(r, &mut game_part) }?;
        return Ok(());
    }
    if place == 7 {
        use crate::race::show_frame::{game_part, show_frame};
        *r = unsafe { show_frame(r, &mut game_part) }?;
        return Ok(());
    }
    crate::calls::call(rva(CALLED[place as usize]), r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    // The thunk is ungated (a frame is long) and the Rust frame calls the Rust versions
    // directly, past every gate: one gate per frame, so that a `selftest`'s pause waits for
    // the frame under way and holds the next (as the audio thread's turns).
    crate::framework::hook_enter();
    let done = race_frame(r, &mut game_part);
    crate::framework::hook_leave();
    let left = done?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
