//! One frame of the race. The window's messages are pumped; quitting ends it there. With the
//! window the active one and not lost: the 3D scene begun, drawn (`race::draw_race_scene`) and
//! ended, the race's sounds, the replay's screenshot, the frame shown.

use crate::errors::Refused;
use crate::race::owner::Race;

/// Takes and returns the value the frame's calls leave on; only `drive::quit_question` reads
/// it.
pub fn race_frame(
    race: &Race,
    o: crate::owners::Owners<'static>,
    eax: u32,
) -> Result<u32, Refused> {
    race_frame_over(race, o, eax, &|| {})
}

/// `race_frame`, with `over` run after the scene is drawn and before the frame is shown: what it
/// writes on the screen's pixels lies over the scene.
pub fn race_frame_over(
    race: &Race,
    o: crate::owners::Owners<'static>,
    eax: u32,
    over: &dyn Fn(),
) -> Result<u32, Refused> {
    let system = o.system;
    let mut eax = eax;
    crate::platform::pump_messages_now(o);
    if system.session.flags.quitting.is_set() {
        // The game is closing. In the game its shutdown thread (`threading::request_quit`)
        // ends the program meanwhile; the standalone, which has none, ends it here.
        crate::standalone::leave_on_close(o);
    }
    if system.session.flags.window_active.get() != 0 && system.window_loop().lost.get() != 1 {
        use crate::debug::timing::{Stage, start, stop};
        let begun = start();
        // `scene::begin_scene` never loses the frame here.
        crate::scene::begin_scene::begin_scene(race, &o.gfx.d3d, o.scene.scratch.game_scratch());
        { crate::race::draw_race_scene::draw_race_scene(race, o) }?;
        crate::scene::end_scene::end_scene(&o.gfx.d3d);
        over();
        stop(Stage::Scene, begun);
        // The frame's sound tick is in `draw_race_scene`.
        { crate::race::replay_screenshot::replay_screenshot(race, o.gfx, o.system) }?;
        crate::race::load_times::first_frame();
        #[cfg(feature = "remote")]
        crate::remote::shot_frame_drawn(race, &o.system.session.picture_clock);
        let shown = { crate::race::show_frame::show_frame(race, o) }?;
        eax = shown.eax;
    }
    // A frame that could not be drawn or shown is skipped: the race goes on and the next frame
    // tries again (the race's results never depend on what was drawn).
    Ok(eax)
}
