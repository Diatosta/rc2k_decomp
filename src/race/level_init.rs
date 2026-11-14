//! The small steps of a stage's level initialisation (`race::loadlevel_init`). The course's memory
//! is counted as 0x1300000 bytes on `CourseLists.course_memory_list`: the course file's room
//! and, as `course_info`, 0x400000 bytes where the texture and mark info files are read.

use crate::cells::Store;
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::trap::Trap;
use crate::scene::file_numbers_store::texts;
use crate::session::TextPlace;

const FIRST_PART: u32 = rc2k_core::race::loading::FIRST_PART as u32;
const INFO_PART: u32 = 0x40_0000;
/// The path strings the course's number is written into: the tens' and the units' places.
pub fn number_places(
    p: &'static crate::race::level_paths_store::LevelPaths,
) -> [crate::place::BytePlace; 6] {
    use crate::place::BytePlace;
    [
        BytePlace::from(&p.edit_path).add(0xD),
        BytePlace::from(&p.cdat_src).add(0xD),
        BytePlace::from(&p.cdat_files).add(0x12),
        BytePlace::from(&p.cdat_src_copy).add(0xD),
        BytePlace::from(&p.localdat_src).add(0xD),
        BytePlace::from(&p.localdat_files).add(0x12),
    ]
}
const TEXTURES: u32 = 0x100;

/// Counts the course's memory (the file's room and the course info) on `list` and takes the course
/// info (`race::course_places::INFO`).
pub fn allocate_course_memory(
    course: &Course,
    list: &crate::memory::ListCells,
) -> Result<(), Refused> {
    course.lists.get().course_bytes_total.set(
        course
            .lists
            .get()
            .course_bytes_total
            .get()
            .wrapping_add(FIRST_PART + INFO_PART),
    );
    crate::memory::owned::count_on(list, FIRST_PART + INFO_PART)?;
    course
        .lists
        .get()
        .course_info
        .set(crate::race::course_places::take_info(INFO_PART)?);
    Ok(())
}

/// Reads `files\a\textinfo`, `files\m\markinfo` and `files\b\textinfo` into the course info
/// (`CourseInfo` lays them out and makes their finish) and takes it into the course info table.
/// Refuses no course info.
pub fn load_texture_info(
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use rc2k_core::race::stage_files::{CourseInfo, MARK_INFO, TEXTURE_INFO_A, TEXTURE_INFO_B};
    let file_numbers = &scene.file_numbers();
    if course.lists.get().course_info.get() == 0 {
        return Err(BadInput("no course info to read the info files into").into());
    }
    let mut info = CourseInfo::new();
    for (texts_at, part) in [
        (texts::TEXTURE_INFO_A, TEXTURE_INFO_A),
        (texts::MARK_INFO, MARK_INFO),
        (texts::TEXTURE_INFO_B, TEXTURE_INFO_B),
    ] {
        let path = file_numbers.text_at(texts_at).text("a course info path")?;
        let mut file = vec![0u8; part.1];
        let read = crate::files::read_whole_file::read_whole_file_len(&path, &mut file)?;
        info.lay_file(part, &file[..read])?;
    }
    info.finish_texture_info();
    crate::race::course_places::set_info(info.bytes())?;
    Ok(())
}

/// Writes the course's two digits (tens, units, from the low byte divided by 10) into the nine
/// path strings (the stage's files) that hold the course's number.
pub fn set_course_number(
    eax: u32,
    race: &'static Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    // Logic in `rc2k_core::race::stage::two_digits`.
    let [tens, units] = rc2k_core::race::stage::two_digits(eax as u8);
    for place in number_places(race.course.level_paths()) {
        place.write(&[tens, units], "a path's course number")?;
    }
    for text in [
        texts::NUMBER_PLACE_A,
        texts::NUMBER_PLACE_B,
        texts::NUMBER_PLACE_C,
    ] {
        scene.file_numbers().text_at(text).write_text_at(
            0,
            &[tens, units],
            "a path's course number",
        )?;
    }
    Ok(())
}

/// Reads the named file, 0x2000 bytes at most, into the error list.
pub fn load_error_list(frame: &Frame, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    crate::files::read_whole_file::read_whole_file_plain_bytes(
        &scene
            .file_numbers()
            .text_at(texts::ERROR_LIST_PATH)
            .text("the error list's path")?,
        &frame.error_list().list,
    )?;
    Ok(())
}

/// Builds the texture infos: nothing without course info; else for each of the 256 textures, from
/// its kind k (`texture_kinds`) and the info's record at k * 0x80, a record of 0x40
/// (`rc2k_core::race::stage_files::texture_infos`). Refuses a kind whose record lies past the
/// info, before any is written.
pub fn build_texture_infos(race: &Race) -> Result<(), Refused> {
    use rc2k_core::race::stage_files::{INFO_SIZE, KIND_RECORD, texture_infos};
    let course = &race.course;
    if course.lists.get().course_info.get() == 0 {
        return Ok(());
    }
    let kinds: [u32; TEXTURES as usize] =
        std::array::from_fn(|n| course.stage_prefs.get().texture_kinds[n].get());
    let trap = Trap::new();
    let infos = texture_infos(&kinds, |kind| {
        if kind as usize >= INFO_SIZE / KIND_RECORD {
            return None;
        }
        trap.keep_read(
            crate::race::course_places::info_bytes(kind * KIND_RECORD as u32, KIND_RECORD)
                .map(Some),
        )
    });
    trap.finish()?;
    for (n, t) in infos?.iter().enumerate() {
        let out = &race.polygon_tables().texture_infos[n].words;
        out[9].set(t.row);
        out[8].set(t.group);
        for (k, &value) in t.values.iter().enumerate() {
            out[4 + k].set(value);
        }
        // The record's byte at +0x1C and three at +0x2C are bytes inside their dwords.
        out[7].store(0, &[t.byte])?;
        out[11].store(0, &t.bytes)?;
        for (k, &value) in t.first.iter().enumerate() {
            out[k].set(value);
        }
        out[10].set(t.marked);
    }
    Ok(())
}

/// Resets `grid_x` and `grid_z`, the course grid's corner (each view keeps its own,
/// `race::view_grid`), so every view makes its grid again.
pub fn reset_refresh() {
    crate::race::view_grid::reset_corners();
}

/// The checkpoints the split tables list: of the table entries 1 .. `last_checkpoint`, those whose
/// face has its byte +0x26 set. None with fewer than two checkpoints. Reads the loaded course
/// and writes nothing. Refused: an end below 2, or more found than the list's 0x201 (a real
/// stage's end is past 0x201).
pub fn listed_checkpoints(race: &Race) -> Result<Option<Vec<u32>>, Refused> {
    let shared = &race.shared;
    if (shared.vars.get().checkpoint_count.get().wrapping_sub(2) as i32) < 0 {
        return Ok(None);
    }
    let end = shared.vars.get().last_checkpoint.get();
    let table = &race.checkpoints().checkpoint_faces;
    let trap = Trap::new();
    let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
        // n < `last_checkpoint`, one less than the chain's length, which `race::best_line`
        // wrote into the table's 5146 entries: inside the table.
        let face = trap.keep_read(
            table
                .get(n as usize)
                .map(crate::session::PlainDword::get)
                .ok_or(BadInput("a checkpoint past the checkpoint table")),
        );
        trap.keep_read(crate::race::course_places::face_field::<u8>(
            race.course.reads(),
            face,
            0x26,
        )) != 0
    })?;
    trap.finish()?;
    Ok(Some(listed))
}

/// Lists the checkpoints (`listed_checkpoints`) in the split tables and sets
/// `split_colour_count` to their count - 1 (0 with fewer than two checkpoints). The list holds
/// 0x101 entries but may run on for up to 0x201, over the stage's numbers that follow it (0x100
/// entries: the two fields end together at 0x201); such a stage is refused later by
/// `race::init_overlay` (more than 0x40 parts).
pub fn init_checkpoints(race: &Race) -> Result<(), Refused> {
    let tables = race.physics.split_tables();
    let Some(listed) = listed_checkpoints(race)? else {
        tables.split_colour_count.set(0);
        return Ok(());
    };
    for (k, &n) in listed.iter().enumerate() {
        let entry = tables
            .checkpoints
            .get(k)
            .or_else(|| tables.stage_numbers.get(k - tables.checkpoints.len()));
        entry
            .ok_or(BadInput(
                "a split past the split list and the stage's numbers",
            ))?
            .set(n);
    }
    tables
        .split_colour_count
        .set((listed.len() as u32).wrapping_sub(1));
    Ok(())
}
