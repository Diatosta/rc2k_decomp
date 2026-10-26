//! More of `race::loadlevel_init`'s steps (IDA, 2026-10-16): "load prefs" (the stage's
//! `localdat`, 0x206C bytes at 0x608CB4, which hold `g_textureKinds` at +0x100 and the
//! lists below), "skidmarks init", "render3d loadlightsdat" and "getmatcrash".

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::car_store::CAR;
use crate::pointers::GamePtr;
use crate::race::matrices_store::MATRICES;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::object_store::OBJECT_WORK;

// The stage's localdat is `STAGE_PREFS.prefs_head` (0x608CB4).
pub const PREFS_LEN: u32 = 0x206C;
/// `files\levels\levelxx\localdat`, `srclvls\levelxx\localdat` (the xx the course's).
fn prefs_paths() -> [usize; 2] {
    let p = &crate::race::level_paths_store::LEVEL_PATHS;
    [
        p.localdat_files.addr() as usize,
        p.localdat_src.addr() as usize,
    ]
}
pub const LIGHTS_LEN: u32 = 0x1320;
pub const LIGHT_RECORDS: u32 = rc2k_core::race::stage_objects::LIGHT_RECORDS as u32;

/// `g_textureKinds`.
fn kind(n: u32) -> u32 {
    STAGE_PREFS.texture_kinds[n as usize].get()
}

/// `dword_60A67C`: 0x40 kinds (>> 4) swapped in.
fn swapped(n: u32) -> u32 {
    STAGE_PREFS.picture_lists_b[n as usize].get()
}

/// `race::swap_stage_texture` (0x439B03): for the stages of the core's `TEXTURE_SWAPS`,
/// the first of the first 0xFF texture kinds that is the stage's old kind becomes its new
/// one; then, unless the new kind is in the swapped list already, it goes >> 4 into the
/// list's first free place of 0x40 (the test is made on the kind before the shift:
/// section 16).
pub fn swap_stage_texture() {
    // Converted: `rc2k_core::race::stage::swap_texture`.
    use rc2k_core::race::stage::{Swap, swap_texture};
    let stage: u32 = RACE_VARS.stage_number.bits();
    let ((_, new), swap) = swap_texture(stage, kind, swapped);
    let swapped_kind = match swap {
        Swap::NotThisStage | Swap::NotFound => return,
        Swap::Listed { kind, .. } | Swap::Full { kind } => kind,
        Swap::Added { kind, at } => {
            STAGE_PREFS.picture_lists_b[at as usize].set(new >> 4);
            kind
        }
    };
    STAGE_PREFS.texture_kinds[swapped_kind as usize].set(new);
}

/// `race::tweak_stage` (0x439AAB): for the stages 0x15, 0x3E and 0x36, `dword_60A564` ..
/// `dword_60A56C` 0xFF.
pub fn tweak_stage() {
    {
        // Converted: `rc2k_core::race::stage::stage_tweaked`.
        if rc2k_core::race::stage::stage_tweaked(RACE_VARS.stage_number.bits()) {
            crate::session::write_bits(&STAGE_PREFS.car_light_table, [0xFFu32; 3]);
        }
    }
}

/// `race::clear_prefs_qword` (0x465381): with `dword_71C800` set, `qword_609344` 0.
pub fn clear_prefs_qword() {
    {
        if RACE_WORK.no_precipitation.get() != 0 {
            crate::session::write_bits(&STAGE_PREFS.blit_cleared, [0u32; 2]);
        }
    }
}

/// `files::read_whole_file` inside `pusha`.
fn read(path: usize, buffer: usize, count: u32) -> Result<(), Refused> {
    crate::race::level_init::read(path as u32, buffer as u32, count)
}

/// `race::load_prefs` (0x445698): `files\levels\levelxx\localdat`, then
/// `srclvls\levelxx\localdat`, each 0x206C bytes at most into 0x608CB4; then
/// `swap_stage_texture`, `tweak_stage`, `clear_prefs_qword`.
pub fn load_prefs() -> Result<(), Refused> {
    for path in prefs_paths() {
        read(path, STAGE_PREFS.prefs_head[0].addr() as usize, PREFS_LEN)?;
    }
    swap_stage_texture();
    tweak_stage();
    clear_prefs_qword();
    Ok(())
}

/// `race::reset_skid_slots` (0x4585BA): the object's (esi) four dwords at +0xF48 -1.
pub fn reset_skid_slots(object: u32) -> Result<(), BadInput> {
    GamePtr::new(object + 0xF48).fill(0xFF, 16, "a skid object's slots")
}

/// `race::init_skidmarks` (0x45857D): the two skid objects' slots, the 0x400 marks' first
/// two words -1, `dword_645B10` 0.
pub fn init_skidmarks() -> Result<(), BadInput> {
    {
        // The skid objects are car blocks 0 and 1.
        for n in 0..2 {
            reset_skid_slots(crate::race::blocks_store::block_addr(n))?;
        }
        let marks = crate::race::shadow_skids_store::SHADOW_SKIDS
            .skid_links
            .addr();
        for n in 0..0x400u32 {
            GamePtr::new(marks + 8 * n).write(u32::MAX, "the skid links")?;
        }
        crate::race::shadow_skids_store::SHADOW_SKIDS
            .skid_next
            .set(0);
    }
    Ok(())
}

/// `race::copy_lights_data` (0x438B03): 0x22 records of 0x90 bytes from the lights' data
/// to +0xA4 of the 0x134-byte records at 0x5C906C.
pub fn copy_lights_data() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::stage_objects::lay_lights`.
    use rc2k_core::race::stage_objects::{LIGHT_RECORD, lay_lights};
    let (data, records) = (
        crate::race::lights_store::LIGHTS_DATA.lights.addr(),
        crate::race::light_records_store::LIGHT_RECORDS.record_addr(0),
    );
    let source = GamePtr::new(data).read_bytes(LIGHTS_LEN as usize, "the lights' data")?;
    let mut laid = GamePtr::new(records)
        .read_bytes(LIGHT_RECORDS as usize * LIGHT_RECORD, "the light records")?;
    lay_lights(&source, &mut laid);
    GamePtr::new(records).write_bytes(&laid, "the light records")
}

/// `race::load_lights_data` (0x438AE5): `files\s\data\ldata` (0x1320 bytes at most) into
/// the lights' data, then `copy_lights_data`.
pub fn load_lights_data() -> Result<(), Refused> {
    read(
        CAMERA_WORK.lights_path[0].addr() as usize, // "files\s\data\ldata"
        crate::race::lights_store::LIGHTS_DATA.lights.addr() as usize,
        LIGHTS_LEN,
    )?;
    copy_lights_data()?;
    Ok(())
}

/// `race::get_mat_crash` (0x43D43E): the three angles (`dword_71A99C`, `dword_71A998`,
/// `dword_71A9A0`, each >> 16) kept at `dword_63503C` .., the x, y and z turns made of
/// them, Z x X into the lit matrix, that x Y into `dword_71A08C`.
pub fn get_mat_crash() {
    use crate::scene::matrix;
    // Converted: `rc2k_core::race::stage_objects::crash_turns` (the matrices the
    // scene's).
    let turns = rc2k_core::race::stage_objects::crash_turns([
        CAR.car_heading.get(),
        CAR.car_pitch.get(),
        CAR.car_roll.get(),
    ]);
    crate::session::write_bits(&OBJECT_WORK.angles, turns);
    matrix::set_turn_x(turns[0]);
    matrix::set_turn_y(turns[1]);
    matrix::set_turn_z_16(turns[2]);
    matrix::multiply_matrices(
        MATRICES.turn_z_matrix[0].addr() as usize,
        MATRICES.turn_x_matrix[0].addr() as usize,
        MATRICES.lit_matrix[0].addr() as usize,
    );
    matrix::multiply_matrices(
        MATRICES.lit_matrix[0].addr() as usize,
        MATRICES.turn_y_matrix[0].addr() as usize,
        CAR.crash_matrix[0].addr() as usize,
    );
}
