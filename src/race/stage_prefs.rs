//! More of `race::loadlevel_init`'s steps: "load prefs" (the stage's `localdat`, 0x206C bytes,
//! which hold `texture_kinds` at +0x100 and the picture lists), "skidmarks init", "render3d
//! loadlightsdat" and "getmatcrash".

use crate::cells::Store;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

// The stage's `localdat` fills the whole of `race().course.stage_prefs` (0x206C bytes).
/// `files\levels\levelxx\localdat`, `srclvls\levelxx\localdat` (the xx the course's).
fn prefs_paths(course: &Course) -> [&dyn Store; 2] {
    let p = course.level_paths();
    [&p.localdat_files, &p.localdat_src]
}
pub const LIGHT_RECORD_COUNT: u32 = rc2k_core::race::stage_objects::LIGHT_RECORDS as u32;

/// `texture_kinds`.
fn kind(race: &Race, n: u32) -> u32 {
    race.course.stage_prefs.get().texture_kinds[n as usize].get()
}

/// `picture_lists_b`: 0x40 kinds (>> 4) swapped in.
fn swapped(race: &Race, n: u32) -> u32 {
    race.course.stage_prefs.get().picture_lists_b[n as usize].get()
}

/// `race::swap_stage_texture`: for the stages of the core's `TEXTURE_SWAPS`, the first of the first
/// 0xFF texture kinds that is the stage's old kind becomes its new one. Unless the new kind is in
/// the swapped list already, it goes >> 4 into the list's first free place of 0x40 (the test is
/// made on the kind before the shift: a slip kept as is).
pub fn swap_stage_texture(race: &Race) {
    let course = &race.course;
    use rc2k_core::race::stage::{Swap, swap_texture};
    let stage: u32 = race.shared.vars.get().stage_number.get();
    let ((_, new), swap) = swap_texture(stage, |n| kind(race, n), |n| swapped(race, n));
    let swapped_kind = match swap {
        Swap::NotThisStage | Swap::NotFound => return,
        Swap::Listed { kind, .. } | Swap::Full { kind } => kind,
        Swap::Added { kind, at } => {
            course.stage_prefs.get().picture_lists_b[at as usize].set(new >> 4);
            kind
        }
    };
    course.stage_prefs.get().texture_kinds[swapped_kind as usize].set(new);
}

/// `race::tweak_stage`: for the stages 0x15, 0x3E and 0x36, sets `StagePrefs.car_light_table` (3
/// dwords) to 0xFF.
pub fn tweak_stage(race: &Race) {
    let course = &race.course;
    {
        if rc2k_core::race::stage::stage_tweaked(race.shared.vars.get().stage_number.get()) {
            crate::session::write_bits(&course.stage_prefs.get().car_light_table, [0xFFu32; 3]);
        }
    }
}

/// `race::clear_prefs_qword`: with `no_precipitation` set, clears `blit_cleared`.
pub fn clear_prefs_qword(race: &Race) {
    let course = &race.course;
    {
        if race.shared.work.get().no_precipitation.get() != 0 {
            crate::session::write_bits(&course.stage_prefs.get().blit_cleared, [0u32; 2]);
        }
    }
}

/// `race::load_prefs`: reads `files\levels\levelxx\localdat`, then `srclvls\levelxx\localdat`,
/// each 0x206C bytes at most, into the prefs; then runs `swap_stage_texture`, `tweak_stage` and
/// `clear_prefs_qword`.
pub fn load_prefs(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    for path in prefs_paths(course) {
        course
            .stage_prefs
            .get()
            .read_file(&crate::files::cstring(path, "a prefs path")?)?;
    }
    swap_stage_texture(race);
    tweak_stage(race);
    clear_prefs_qword(race);
    Ok(())
}

/// `race::reset_skid_slots`: sets the object's four skid slots to -1.
pub fn reset_skid_slots(object: &crate::physics::car_store::Car) {
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

/// `race::copy_lights_data`: copies 0x22 records of 0x90 bytes from the lights' data to +0xA4 of
/// the 0x134-byte light records.
pub fn copy_lights_data(race: &Race) -> Result<(), BadInput> {
    use rc2k_core::race::stage_objects::{LIGHT_RECORD, lay_lights};
    let records = &race.light_records().records;
    let source = race.course.lights_data().lights.read();
    let mut laid = vec![0; LIGHT_RECORD_COUNT as usize * LIGHT_RECORD];
    Store::load(records, 0, &mut laid)?;
    lay_lights(&source, &mut laid);
    Store::store(records, 0, &laid)
}

/// `race::load_lights_data`: reads `files\s\data\ldata` (0x1320 bytes at most) into the lights'
/// data, then runs `copy_lights_data`.
pub fn load_lights_data(race: &Race) -> Result<(), Refused> {
    crate::files::read_whole_file::read_whole_file_plain_bytes(
        &crate::files::cstring(&race.physics.camera().lights_path, "the lights data path")?,
        &race.course.lights_data().lights,
    )?;
    copy_lights_data(race)?;
    Ok(())
}

/// Keeps the three angles (`car_pitch`, `car_heading`, `car_roll`, each shifted right by 16) in
/// `angles`, makes the x, y and z turns of them, and multiplies Z x X into the lit matrix and that
/// x Y into `crash_matrix`.
pub fn get_mat_crash(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
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
    crate::session::write_bits(&scene.object_work().angles, turns);
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
