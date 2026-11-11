//! `drive::race_frame` (0x4541BF): one frame of the race. The window's messages
//! (`system::pump_messages`); quitting (`g_quitting` 1) ends it there. With the window the active
//! one and `dword_518380` not 1: the 3D scene begun, drawn (`race::draw_race_scene`) and ended
//! (`scene::end_scene`), the race's sounds, the replay's screenshot (`race::replay_screenshot`),
//! the frame shown; made: carry clear.
//!
//! Callers: `drive::race_loop`, `quit_question`, `pause_until_key`, `finish_run`,
//! `race::play_movie_loop`. The drawing, the sound tick, the screenshot and the showing are their
//! Rust versions; the window, the scene's begin and end and the restore through the game.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::system;

const RESTORE: usize = 0x405B10; // 0x405B10: the surfaces restored (the platform's)

/// Takes and returns the value the frame's calls leave on (the original's register); only
/// `drive::quit_question` reads it.
pub fn race_frame(race: &Race, eax: u32) -> Result<u32, Refused> {
    let input = crate::input::owner::input();
    let system = system();

    let mut eax = eax;
    crate::platform::pump_messages_now();
    if system.session.flags.quitting.is_set() {
        // The game is closing. In the game its shutdown thread (`threading::request_quit`)
        // ends the program meanwhile; the standalone, which has none, ends it here.
        crate::standalone::leave_on_close();
    }
    if system.session.flags.window_active.get() != 0 && system.session.flags.lost.get() != 1 {
        use crate::debug::timing::{Stage, start, stop};
        let begun = start();
        // `scene::begin_scene` never loses the frame here.
        crate::scene::begin_scene::begin_scene();
        {
            let _step = crate::write_watch::step("draw");
            crate::race::draw_race_scene::draw_race_scene(race, input, system)
        }?;
        crate::scene::end_scene::end_scene();
        stop(Stage::Scene, begun);
        // The original's sound tick here (tick 0x12) never found a turn due: the frame's one
        // is in `draw_race_scene`.
        { crate::race::replay_screenshot::replay_screenshot() }?;
        #[cfg(feature = "remote")]
        crate::remote::shot_frame_drawn();
        let shown = {
            let _step = crate::write_watch::step("show");
            crate::race::show_frame::show_frame(race)
        }?;
        eax = shown.eax;
        if !shown.lost {
            return Ok(eax);
        }
    }
    if system.session.flags.quitting.is_set() {
        return Ok(eax);
    }
    // The surfaces restored: never here (the window is always active, nothing is lost).
    crate::standalone::not_ported(RESTORE, "drive::race_frame")
}
