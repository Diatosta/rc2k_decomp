//! `race::draw_mirror_view` (0x454806): the mirror's view, unless `dword_64BB18` or `in_credits`
//! is set or `scene_set_up` (`scene_set_up`, always 1 once a scene is set up: it never stops the mirror) is 0: `dword_63DC58` kept in `view_value_kept`;
//! `view_scale` saved, made `k256`'s value (the mirror's own view variables,
//! `ViewScratch::vars`, taken first); the mirror's window (`race::mirror_view_window`),
//! `dword_63DC58` back from `view_value_kept`, the backdrop (`race::draw_view_backdrop`), the scene
//! (`race::draw_view_scene`); `view_scale` put back; the full view's window again
//! (`race::full_view_window_2`, now the front end's window: `race::draw_race_scene`). Returns the
//! scene's output (`ViewOutput`: its view value for its block; its EAX weight goes nowhere, the
//! race's is the first player's view's).
//!
//! The original also set `g_gridReach` to 2 and put it back after. Only the mirror's
//! `scene::set_view_window` read the 2 (the fog's `race::draw_distance_settings` then sets the
//! level's reach), so the window takes it as an argument (`MIRROR_REACH`). What was put back
//! was the level's reach whenever a view before the mirror had set up its fog; it can differ
//! only on a stage's first frame with the main view covered (`notes/frame_plan.md`, F4).

use crate::errors::Refused;
use crate::race::owner::Race;

/// With `RC2K_MIRROR_ALWAYS` (a check): the mirror drawn in every view, replays' included,
/// whatever the car's options and the mirror switch say, so that the mirror's drawing can be
/// compared over a replay.
pub fn always() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_MIRROR_ALWAYS").is_some())
}

pub fn draw_mirror_view(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<Option<crate::race::draw_view_scene::ViewOutput>, Refused> {
    view.take_vars(race);
    if race.replay.finish.no_place.get() != 0
        || race.shared.work.in_credits.get() != 0
        || (race.shared.switches.scene_set_up.get() == 0 && !always())
    {
        return Ok(None);
    }
    // The main view's depth (`RACE_VIEW.view_depth[1]` as its `scene::set_view_window` left
    // it: the level's grid reach times 0x20), worked out here, not read from the main view's
    // window while its own part may be writing it.
    view.course
        .view_value_kept
        .set(race.course.stage_prefs.grid_reach.get().wrapping_mul(0x20));
    let saved = view.vars.view_scale.bits();
    view.vars
        .view_scale
        .set_bits(race.course.numbers.k256.bits());
    let drawn = (|| {
        crate::race::frame_views::mirror_view_window(race, scene, system, view)?;
        view.polygon.window.view_depth[1].set(view.course.view_value_kept.get());
        crate::race::frame_flush::draw_view_backdrop(race, scene, view);
        crate::race::draw_view_scene::draw_view_scene(race, scene, view)
    })();
    view.vars.view_scale.set_bits(saved);
    // (The original's full window again after the mirror is the front end's: `race::draw_race_scene`.)
    drawn.map(Some)
}
