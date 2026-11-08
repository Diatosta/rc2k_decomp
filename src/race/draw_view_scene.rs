//! `race::draw_view_scene` (0x4557AB): a view's 3D scene. With `unk_71AF80` set (a reading:
//! the view covered) only `race::draw_view_tint`. Else `dword_635014`, `dword_635018` 0 and
//! in turn, with the race's sound tick (`sound::race_sound_tick`, eax 0x14 .. 0x1A in the
//! original: the Rust tick takes none) between the groups: the car lights' modes, the point
//! records cleared, the fog, the camera's view, the sky and the draw batch flushed, the
//! view's light; the lamps reset, `dword_63DB6C` 0, the course's view prepared, its visible
//! faces, the EAX environment, the lamps gathered and wanted; the sort entries sorted and
//! drawn, the race's slots; the cars' shadows, the skidmarks, the car lights; the depth
//! entries sorted and drawn, the batch flushed, the view's value stored.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::scene::faces_store::SORT_LISTS;

fn sort_list() -> crate::pointers::GamePtr {
    crate::scene::faces_store::list_at(core::mem::offset_of!(
        crate::scene::faces_store::SortLists,
        sort_entries
    ))
}

fn depth_list() -> crate::pointers::GamePtr {
    crate::scene::faces_store::list_at(core::mem::offset_of!(
        crate::scene::faces_store::SortLists,
        depth_entries
    ))
}

/// The Rust version (see the header).
pub fn draw_view_scene(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    use crate::race::frame_draw::*;
    if race.shared.car.view_covered.get() != 0 {
        return draw_view_tint(race, scene);
    }
    // dword_635014, dword_635018.
    for cell in &scene.drawing.object_work.view_cleared {
        cell.set(0);
    }
    car_light_modes(race)?;
    clear_point_records(race, scene)?;
    set_up_fog(race, scene)?;
    camera_view(race, scene)?;
    draw_sky(race, scene)?;
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    view_light(race, scene)?;
    reset_lamps(race, scene)?;
    race.shared.view.view_lamps.set_bits(0u32);
    prepare_course_view(race, scene)?;
    gather_visible_faces(race, scene)?;
    eax_environment(race, scene)?;
    gather_lamps(race, scene)?;
    lamps_wanted(&race.shared)?;
    sort_entries(&race.shared.view.sort_entry_count, &SORT_LISTS.sort_entries)?;
    let c = crate::moved::census::before();
    let begun = crate::debug::timing::start();
    draw_entries(
        race,
        scene,
        &race.shared.view.sort_entry_count,
        &SORT_LISTS.sort_entries,
        sort_list(),
    )?;
    crate::debug::timing::stop(crate::debug::timing::Stage::CourseEntries, begun);
    crate::moved::census::after("sorted", c);
    draw_race_slots(race, scene)?;
    crate::race::car_shadow::cast_car_shadows(race, scene)?;
    crate::race::skidmarks::draw_skidmarks_if_on(race, scene)?;
    car_lights(race, scene)?;
    sort_entries(
        &race.shared.view.depth_entry_count,
        &SORT_LISTS.depth_entries,
    )?;
    let c = crate::moved::census::before();
    let begun = crate::debug::timing::start();
    draw_entries(
        race,
        scene,
        &race.shared.view.depth_entry_count,
        &SORT_LISTS.depth_entries,
        depth_list(),
    )?;
    crate::debug::timing::stop(crate::debug::timing::Stage::DepthEntries, begun);
    crate::moved::census::after("depth", c);
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    store_view_value(race)
}
