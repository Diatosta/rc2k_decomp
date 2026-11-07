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
//! versions; the window, the scene's begin and end and the restore through the game.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::system;

const RESTORE: usize = 0x405B10; // sub_405B10: the surfaces restored (the platform's)

/// The Rust version (see the header): `eax` in, the eax its calls leave out (`mov eax,
/// 12h` and `race::show_frame` when the frame is drawn, else what the window's messages
/// and the restore leave of it). Only `drive::quit_question` reads it (into
/// `input::yes_no_key`, liveness: 0x44520A); its flags and other registers are read by
/// no caller (only DF live after the other four calls).
pub fn race_frame(race: &Race, eax: u32) -> Result<u32, Refused> {
    let system = system();

    let frame = &race.frame;
    // The calls below read no register at entry (liveness: `pump_messages`,
    // `race_sound_tick`, `replay_screenshot`, `show_frame`, `sub_405B10`): only eax runs
    // from call to call, and only `show_frame`'s carry is read.
    let mut eax = eax;
    crate::platform::pump_messages_now();
    if system.session.flags.quitting.get() == 1 {
        // The game is closing. In the game its shutdown thread (`threading::request_quit`)
        // ends the program meanwhile; the standalone, which has none, ends it here.
        crate::standalone::leave_on_close();
    }
    if system.session.flags.window_active.get() != 0 && system.session.flags.lost.get() != 1 {
        // `scene::begin_scene` leaves the carry clear: the frame is never lost there.
        use crate::debug::timing::{Stage, start, stop};
        let begun = start();
        crate::scene::begin_scene::begin_scene();
        { crate::race::draw_race_scene::draw_race_scene(race) }?;
        crate::scene::end_scene::end_scene();
        stop(Stage::Scene, begun);
        // eax 0x12 here in the original; read by nothing.
        let begun = start();
        crate::sound::race_sound_tick::race_sound_tick(frame)?;
        stop(Stage::SoundTick, begun);
        { crate::race::replay_screenshot::replay_screenshot() }?;
        #[cfg(feature = "remote")]
        crate::remote::shot_frame_drawn();
        let shown = { crate::race::show_frame::show_frame(race) }?;
        eax = shown.eax;
        if !shown.lost {
            return Ok(eax);
        }
    }
    if system.session.flags.quitting.get() == 1 {
        return Ok(eax);
    }
    // The surfaces restored: never here (the window is always active, nothing is lost).
    crate::standalone::not_ported(RESTORE, "drive::race_frame")
}
