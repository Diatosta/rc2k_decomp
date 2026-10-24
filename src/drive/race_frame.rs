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
use crate::framework::{Regs, rva};
use crate::session::FLAGS;

const PUMP: usize = 0x410535; // system::pump_messages (the platform's)
const RESTORE: usize = 0x405B10; // sub_405B10: the surfaces restored (the platform's)

/// The Rust version (see the header): `eax` in, the eax its calls leave out (`mov eax,
/// 12h` and `race::show_frame` when the frame is drawn, else what the window's messages
/// and the restore leave of it). Only `drive::quit_question` reads it (into
/// `input::yes_no_key`, liveness: 0x44520A); its flags and other registers are read by
/// no caller (only DF live after the other four calls).
pub fn race_frame(eax: u32) -> Result<u32, Refused> {
    // The calls below that still take the registers read none at entry (liveness:
    // `pump_messages`, `race_sound_tick`, `replay_screenshot`,
    // `show_frame`, `sub_405B10`): one local scratch, as the original's registers run
    // from call to call. Only `show_frame`'s carry is read.
    let r = &mut Regs {
        eax,
        ..Regs::default()
    };
    crate::calls::call(rva(PUMP), r);
    if FLAGS.quitting.get() == 1 {
        // The game is closing. In the game its shutdown thread (`threading::request_quit`)
        // ends the program meanwhile; the standalone, which has none, ends it here.
        crate::standalone::leave_from_race();
    }
    if FLAGS.window_active.get() != 0 && FLAGS.lost.get() != 1 {
        // `scene::begin_scene` leaves the carry clear: the frame is never lost there.
        crate::scene::begin_scene::begin_scene();
        unsafe { crate::race::draw_race_scene::draw_race_scene() }?;
        crate::scene::end_scene::end_scene();
        r.eax = 0x12;
        crate::sound::race_sound_tick::race_sound_tick()?;
        unsafe { crate::race::replay_screenshot::replay_screenshot() }?;
        let shown = unsafe { crate::race::show_frame::show_frame() }?;
        r.eax = shown.eax;
        if !shown.lost {
            return Ok(r.eax);
        }
    }
    if FLAGS.quitting.get() == 1 {
        return Ok(r.eax);
    }
    r.ebx = FLAGS.lost.get();
    crate::calls::call(rva(RESTORE), r);
    Ok(r.eax)
}
