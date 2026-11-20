//! More of `race::loadlevel_init`'s steps: "load prefs" (the stage's `localdat`, 0x206C bytes,
//! which hold `texture_kinds` and the picture lists), "skidmarks init", "render3d
//! loadlightsdat" and "getmatcrash".

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::memory::cells::Store;
use crate::physics::car_store::Car;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// `files\levels\levelxx\localdat`, `srclvls\levelxx\localdat` (the xx the course's).
fn prefs_paths(course: &Course) -> [&dyn Store; 2] {
    let p = course.level_paths();
    [&p.localdat_files, &p.localdat_src]
}
const LIGHT_RECORD_COUNT: u32 = rc2k_core::race::stage_objects::LIGHT_RECORDS as u32;

/// Loads the stage's preferences: `files\levels\levelxx\localdat`, then `srclvls\levelxx\localdat`
/// laid over what the preferences hold, then the stage's texture swap, tweak and the cleared
/// noise (`StagePrefsFile`), and the result stored.
pub(super) fn load_prefs(race: &Race) -> Result<(), Refused> {
    use rc2k_core::race::stage_files::{PREFS_SIZE, StagePrefsFile};
    let course = &race.course;
    let held = course.stage_prefs.get();
    let mut prefs = StagePrefsFile::from_held(&crate::memory::cells::flatten(held))?;
    for path in prefs_paths(course) {
        let mut file = vec![0u8; PREFS_SIZE];
        let read = crate::files::read_whole_file::read_whole_file_len(
            &crate::files::cstring(path, "a prefs path")?,
            &mut file,
        )?;
        prefs.overlay(&file[..read]);
    }
    let stage = race.shared.vars.get().stage_number.get();
    prefs.swap_stage_texture(stage);
    prefs.tweak_stage(stage);
    prefs.clear_precipitation_range(race.shared.work.get().no_precipitation.get() != 0);
    crate::memory::cells::store_cells(held, 0, prefs.bytes())?;
    Ok(())
}

/// `race::reset_skid_slots`: sets the object's four skid slots to -1.
fn reset_skid_slots(object: &crate::physics::car_store::Car) {
    for slot in &object.skid_slots {
        slot.set(u32::MAX);
    }
}

/// `race::init_skidmarks`: resets the two skid objects' slots, sets the 0x400 links' first two
/// words to -1 and `skid_next` to 0.
pub fn init_skidmarks(race: &Race) -> Result<(), BadInput> {
    let frame = &race.frame;
    {
        // The skid objects are car blocks 0 and 1.
        for n in 0..2 {
            reset_skid_slots(&race.car_blocks().cars[n]);
        }
        let links = &frame.shadow_skids.get().skid_links;
        for n in 0..0x400 {
            links.set_u32(8 * n, u32::MAX);
        }
        frame.shadow_skids.get().skid_next.set(0);
    }
    Ok(())
}

/// `race::copy_lights_data`: copies 0x22 records of 0x90 bytes from the lights' data into the lights' part of
/// the 0x134-byte light records.
pub fn copy_lights_data(race: &Race) -> Result<(), BadInput> {
    use rc2k_core::race::stage_objects::{LIGHT_RECORD, lay_lights};
    let records = &race.light_records().records;
    let source = race.course.lights_table().lights;
    let mut laid = vec![0; LIGHT_RECORD_COUNT as usize * LIGHT_RECORD];
    Store::load(records, 0, &mut laid)?;
    lay_lights(&source, &mut laid);
    Store::store(records, 0, &laid)
}

/// `race::load_lights_data`: reads `files\s\data\ldata` (0x1320 bytes at most) into the lights'
/// data, then runs `copy_lights_data`.
pub(super) fn load_lights_data(race: &Race) -> Result<(), Refused> {
    let lights_path: Vec<u8> = race
        .physics
        .camera()
        .lights_path
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    crate::files::read_whole_file::read_whole_file_len(
        &crate::files::cstring_in(&lights_path)?,
        &mut race.course.lights_table_mut().lights,
    )?;
    copy_lights_data(race)?;
    Ok(())
}

/// Keeps the three angles (`car_pitch`, `car_heading`, `car_roll`, each shifted right by 16) in
/// `angles`, makes the x, y and z turns of them, and multiplies Z x X into the lit matrix and that
/// x Y into `crash_matrix`.
pub fn get_mat_crash(
    engine_tables: &crate::math::engine_tables::EngineTables,
    matrices: &SceneMatrices,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    use crate::scene::matrix;
    let turns = rc2k_core::race::stage_objects::crash_turns([
        car.car_heading.get(),
        car.car_pitch.get(),
        car.car_roll.get(),
    ]);
    scene.object_work_mut().angles = turns;
    matrix::set_turn_x(
        engine_tables,
        &matrices.turn_x_matrix,
        &scene.drawing.scene.get().turn_x_work,
        turns[0],
    );
    matrix::set_turn_y(
        engine_tables,
        &matrices.turn_y_matrix,
        &scene.drawing.scene.get().turn_y_work,
        turns[1],
    );
    matrix::set_turn_z_16(
        engine_tables,
        &matrices.turn_z_matrix,
        &scene.drawing.scene.get().turn_z_work,
        turns[2],
    );
    matrix::multiply_matrices(
        &matrices.turn_z_matrix,
        &matrices.turn_x_matrix,
        &matrices.lit_matrix,
    );
    matrix::multiply_matrices(
        &matrices.lit_matrix,
        &matrices.turn_y_matrix,
        &car.crash_matrix,
    );
}
