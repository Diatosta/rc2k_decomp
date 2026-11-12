//! `race::find_present_courses` (0x4394F8): which of the 100 courses are there: for each, its
//! number in the stage's paths (`race::set_course_number`, `race::set_cdat_number` with 0), its
//! flag (`course_present`[n]`) 0, and 4 bytes of `srclvls\levelNN\cdat00c` (0x60B02B) read into
//! `work_dword` (`files::read_whole_file`, inside two `pusha`); four read (`g_readBytes`), the
//! flag 1; else the same with `files\levels\levelNN\cdat00c.sq` (0x60B07A).
//!
//! Callers: `scene::set_up_lit_engine`, and code at 0x442BF3 (not a function in IDA). Refuses
//! what `files::read_whole_file` refuses (nothing written then).

use crate::errors::Refused;
use crate::files::read_whole_file::read_whole_file_cells;
use crate::files::state::BYTES_READ;
use crate::race::course_data::set_cdat_number;
use crate::race::level_init::set_course_number;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// `srclvls\level01\cdat00c`, `files\levels\level01\cdat00c.sq`.
fn paths(course: &Course) -> [&dyn crate::pointers::Store; 2] {
    let p = &course.level_paths;
    [&p.cdat_src, &p.cdat_files]
}
use rc2k_core::race::stage_objects::COURSES;

/// (nothing written when it refuses).
pub fn find_present_courses(
    race: &'static Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let course = &race.course;
    // Converted: `rc2k_core::race::stage_objects::{COURSES, course_present, PROBE_BYTES}`.
    use rc2k_core::race::stage_objects::{PROBE_BYTES, course_present};
    for n in 0..COURSES {
        set_course_number(n, race, scene)?;
        set_cdat_number(course, 0);
        shared.switches.course_present[n as usize].set(0); // FrameSwitches.course_present[0], 100 flags
        let present = course_present(paths(course).len() as u32, |file| {
            BYTES_READ.set(0);
            let path = paths(course)[file as usize];
            debug_assert_eq!(PROBE_BYTES, 4);
            read_whole_file_cells(
                &crate::files::cstring(path, "a course file's path")?,
                std::slice::from_ref(&scene.drawing.scene.work_dword),
            )?;
            Ok::<u32, Refused>(BYTES_READ.get())
        })?;
        if present {
            shared.switches.course_present[n as usize].set(1);
        }
    }
    Ok(())
}
