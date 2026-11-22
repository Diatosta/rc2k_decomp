//! Draws the mirror's view, unless `Finish.no_place` or `in_credits` is set or `scene_set_up` is 0
//! (it is always 1 once a scene is set up, so it never stops the mirror). The steps: keep
//! the view window's `view_depth[1]` in `view_value_kept`; save `view_scale` and make it `k256`'s value
//! (the mirror's own view variables, `ViewScratch::vars`, are taken first); set the mirror's
//! window (`race::mirror_view_window`); put the view window's `view_depth[1]` back from `view_value_kept`;
//! draw the backdrop (`race::draw_view_backdrop`) and the scene (`race::draw_view_scene`); put
//! `view_scale` back; set the full view's window again (`race::frame_views::full_view_window`, now
//! the front end's window: `race::draw_race_scene`). Returns the scene's output (`ViewOutput`: its
//! view value for its block; its weight goes nowhere, the race's is the first player's view's).
//!
//! The mirror's `scene::set_view_window` is the only reader of a `grid_reach` of 2 (the fog's
//! `race::draw_distance_settings` then sets the level's reach), so the window takes it as an
//! argument (`MIRROR_REACH`) and `grid_reach` is not touched. The one difference: the level's
//! reach that used to be put back could differ only on a stage's first frame with the main view
//! covered.

use crate::errors::Refused;
use crate::race::owner::Race;

/// With `RC2K_MIRROR_ALWAYS` (a check): the mirror drawn in every view, replays' included,
/// whatever the car's options and the mirror switch say, so that the mirror's drawing can be
/// compared over a replay.
pub fn always() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_MIRROR_ALWAYS").is_some())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's owners, its view, scratch and reads, as `DrawCtx::new` takes them"
)]
pub fn draw_mirror_view(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    input: &crate::input::owner::Input,
    view: &crate::race::view_scratch::ViewScratch,
    part: &crate::scene::scratch_store::PartScratch,
    reads: crate::scene::workers::DrawReads<'_>,
) -> Result<Option<crate::race::draw_view_scene::ViewOutput>, Refused> {
    view.take_vars(reads);
    if reads.no_place != 0
        || reads.work.in_credits()
        || (reads.switches.scene_set_up() == 0 && !always())
    {
        return Ok(None);
    }
    // The main view's depth (`view_depth[1]` as its `scene::set_view_window` leaves it:
    // the level's grid reach times 0x20) is worked out here, not read from the main view's window
    // while its own part may be writing it.
    view.course.borrow_mut().view_value_kept =
        reads.stage_prefs.read(|p| p.grid_reach).wrapping_mul(0x20);
    let saved = view.vars.borrow().view_scale;
    view.vars.borrow_mut().view_scale = reads.numbers.k256.get();
    let drawn = (|| {
        crate::race::frame_views::mirror_view_window(race, view, part, reads)?;
        view.polygon.window.borrow_mut().view_depth[1] = view.course.borrow().view_value_kept;
        crate::race::frame_flush::draw_view_backdrop(reads, view, part);
        // A mirror starts from no filter switches (`race::view_scratch::filter_input`: only the
        // first view's slot reads the front end's).
        crate::race::draw_view_scene::draw_view_scene(
            race, scene, input, view, [0; 3], part, reads, None,
        )
    })();
    view.vars.borrow_mut().view_scale = saved;
    // The full window the front end uses after the mirror is set again in `race::draw_race_scene`.
    drawn.map(Some)
}
