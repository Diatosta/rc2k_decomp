//! More of `race::loadlevel_init`'s steps: "load prefs" (the stage's `localdat`, 0x206C bytes,
//! which hold `texture_kinds` and the picture lists), "skidmarks init", "render3d
//! loadlightsdat" and "getmatcrash".

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::memory::session::DrawPlain;
use crate::physics::car_store::Car;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// `files\levels\levelxx\localdat`, `srclvls\levelxx\localdat` (the xx the course's).
fn prefs_paths(course: &Course) -> Result<[Vec<u8>; 2], crate::errors::BadInput> {
    let p = course.level_paths();
    Ok([
        crate::files::cstring_in(&p.localdat_files)?,
        crate::files::cstring_in(&p.localdat_src)?,
    ])
}
/// The bytes of one model's lights' data in the lights' table.
const LIGHT_DATA: usize = 0x90;

/// Loads the stage's preferences: `files\levels\levelxx\localdat`, then `srclvls\levelxx\localdat`
/// laid over what the preferences hold, then the stage's texture swap, tweak and the cleared
/// noise (`StagePrefsFile`), and the result stored.
pub(super) fn load_prefs(race: &Race) -> Result<(), Refused> {
    use rc2k_core::race::stage_files::{PREFS_SIZE, StagePrefsFile};
    let course = &race.course;
    let mut prefs = StagePrefsFile::from_held(&course.stage_prefs.borrow().to_le_bytes())?;
    for path in prefs_paths(course)? {
        let mut file = vec![0u8; PREFS_SIZE];
        let read = crate::files::read_whole_file::read_whole_file_len(&path, &mut file)?;
        prefs.overlay(&file[..read]);
    }
    let stage = race.shared.vars.get().stage_number.get();
    prefs.swap_stage_texture(stage);
    prefs.tweak_stage(stage);
    prefs.clear_precipitation_range(race.shared.work.get().no_precipitation.get() != 0);
    course
        .stage_prefs
        .borrow_mut()
        .set_from_le_bytes(prefs.bytes())?;
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

/// Copies the lights' data (0x90 bytes for each of the 0x22 models) into the lights' part of each
/// model's light record.
pub fn copy_lights_data(race: &Race) -> Result<(), BadInput> {
    let source = race.course.lights_table().lights;
    let mut records = race.frame.light_records.borrow_mut();
    for (record, data) in records
        .records
        .iter_mut()
        .zip(source.as_chunks::<LIGHT_DATA>().0)
    {
        for (word, bytes) in record.lights.iter_mut().zip(data.as_chunks::<4>().0) {
            *word = u32::from_le_bytes(*bytes);
        }
    }
    Ok(())
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
    matrices: &DrawPlain<SceneMatrices>,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    use crate::scene::matrix::{
        TURN_X_BASE, TURN_Y_BASE, TURN_Z_BASE, matrix_of, product, store_matrix, turned_x,
        turned_y, turned_z_16,
    };
    let turns = rc2k_core::race::stage_objects::crash_turns([
        car.car_heading.get(),
        car.car_pitch.get(),
        car.car_roll.get(),
    ]);
    scene.object_work_mut().angles = turns;
    let (turn_x, _) = turned_x(engine_tables, TURN_X_BASE, turns[0]);
    let (turn_y, _) = turned_y(engine_tables, TURN_Y_BASE, turns[1]);
    let (turn_z, _) = turned_z_16(engine_tables, TURN_Z_BASE, turns[2]);
    let lit = {
        let mut kept = matrices.borrow_mut();
        kept.lit_matrix = product(&turn_z, &turn_x, kept.lit_matrix);
        kept.lit_matrix
    };
    let crash = product(&lit, &turn_y, matrix_of(&car.crash_matrix));
    store_matrix(&car.crash_matrix, &crash);
}
