//! `race::draw_mirror_view` (0x454806): the mirror's view, unless `dword_64BB18` or
//! `dword_71C3E0` is set or `dword_621480` (the mirror on, a reading) is 0:
//! `dword_63DC58` kept in `dword_63E170`; `dword_606294` and `g_gridReach` saved, made
//! `flt_5C5E2C`'s value and 2; the mirror's window (`race::mirror_view_window`),
//! `dword_63DC58` back from `dword_63E170`, the backdrop (`race::draw_view_backdrop`), the
//! scene (`race::draw_view_scene`); the two put back; the full view's window again
//! (`race::full_view_window_2`).

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version (see the header).
pub fn draw_mirror_view(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    if race.replay.finish.no_place.get() != 0
        || race.shared.work.in_credits.get() != 0
        || race.shared.switches.other_way.get() == 0
    {
        return Ok(());
    }
    race.course
        .level_grid
        .view_value_kept
        .set_bits(race.shared.view.view_depth[1].bits());
    let saved = (
        race.frame.view_and_records.view_vars.view_scale.bits(),
        race.course.stage_prefs.grid_reach.bits(),
    );
    race.frame
        .view_and_records
        .view_vars
        .view_scale
        .set_bits(race.course.numbers.k256.bits());
    race.course.stage_prefs.grid_reach.set_bits(2u32);
    let drawn = (|| {
        crate::race::frame_views::mirror_view_window(race, scene, system)?;
        race.shared.view.view_depth[1].set_bits(race.course.level_grid.view_value_kept.bits());
        crate::race::frame_flush::draw_view_backdrop(race, scene);
        crate::race::draw_view_scene::draw_view_scene(race, scene)
    })();
    race.course.stage_prefs.grid_reach.set_bits(saved.1);
    race.frame
        .view_and_records
        .view_vars
        .view_scale
        .set_bits(saved.0);
    drawn?;
    crate::race::frame_views::full_view_window(race, false);
    Ok(())
}
