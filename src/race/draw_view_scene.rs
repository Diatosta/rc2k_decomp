//! `race::draw_view_scene` (0x4557AB): a view's 3D scene. With `unk_71AF80` set (a reading:
//! the view covered) only `race::draw_view_tint`. Else `dword_635014`, `dword_635018` 0 and
//! in turn, with the race's sound tick (`sound::race_sound_tick`, eax 0x14 .. 0x1A in the
//! original: the Rust tick takes none) between the groups: the car lights' modes, the point
//! records cleared, the fog, the camera's view, the sky and the draw batch flushed, the
//! view's light; the lamps reset, `dword_63DB6C` 0, the course's view prepared, its visible
//! faces, the EAX environment, the lamps gathered and wanted; the sort entries sorted and
//! drawn, the race's slots; the cars' shadows, the skidmarks, the car lights; the depth
//! entries sorted and drawn, the batch flushed, the view's value stored.
//!
//! In:  nothing (no step reads a register its caller gives: register liveness and the Rust
//!      steps, 2026-10-23; edx's "read" is `fog_table`'s `imul`)
//! Out: nothing its callers (`race::draw_race_scene`, `race::draw_mirror_view`) read

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::race::view_store::RACE_VIEW;
use crate::scene::object_store::OBJECT_WORK;
const SORT_ENTRIES: u32 = 0x9DB730; // g_sortEntries
const DEPTH_ENTRIES: u32 = 0x9D3730; // g_depthEntries

/// The Rust version (see the header).
pub unsafe fn draw_view_scene() -> Result<(), Refused> {
    use crate::race::frame_draw::*;
    use crate::sound::race_sound_tick::race_sound_tick;
    unsafe {
        if CAR.view_covered.get() != 0 {
            return draw_view_tint();
        }
        // dword_635014, dword_635018.
        for cell in &OBJECT_WORK.view_cleared {
            cell.set(0);
        }
        race_sound_tick()?; // 0x14
        car_light_modes()?;
        clear_point_records();
        set_up_fog()?;
        camera_view();
        draw_sky()?;
        crate::race::flush_draw_batch::flush_draw_batch()?;
        view_light()?;
        race_sound_tick()?; // 0x15
        reset_lamps();
        RACE_VIEW.view_lamps.set_bits(0u32);
        prepare_course_view()?;
        gather_visible_faces()?;
        eax_environment();
        gather_lamps()?;
        lamps_wanted()?;
        race_sound_tick()?; // 0x16
        sort_entries(&RACE_VIEW.sort_entry_count, SORT_ENTRIES)?;
        draw_entries(&RACE_VIEW.sort_entry_count, SORT_ENTRIES)?;
        draw_race_slots()?;
        race_sound_tick()?; // 0x17
        crate::race::car_shadow::cast_car_shadows()?;
        crate::race::skidmarks::draw_skidmarks_if_on()?;
        car_lights()?;
        race_sound_tick()?; // 0x18
        sort_entries(&RACE_VIEW.depth_entry_count, DEPTH_ENTRIES)?;
        race_sound_tick()?; // 0x19
        draw_entries(&RACE_VIEW.depth_entry_count, DEPTH_ENTRIES)?;
        race_sound_tick()?; // 0x1A
        crate::race::flush_draw_batch::flush_draw_batch()?;
        store_view_value()
    }
}
