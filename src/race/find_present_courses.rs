//! Finds which of the 100 courses are there: for each, sets its number in the stage's paths
//! (`race::set_course_number`, `race::set_cdat_number` with 0), its flag (`course_present[n]`) to
//! 0, and reads 4 bytes of `srclvls\levelNN\cdat00c` into `work_dword` (`files::read_whole_file`);
//! if four are read (`read_bytes`) the flag is 1; else it tries the same with
//! `files\levels\levelNN\cdat00c.sq`. Called by `scene::set_up_lit_engine`. Refused: what
//! `files::read_whole_file` refuses (nothing written then).

use crate::errors::Refused;
use crate::files::read_whole_file::read_whole_file_cells;
use crate::race::course_data::set_cdat_number;
use crate::race::level_init::set_course_number;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// `srclvls\level01\cdat00c`, `files\levels\level01\cdat00c.sq`.
fn paths(course: &Course) -> [&dyn crate::cells::Store; 2] {
    let p = course.level_paths();
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
    use rc2k_core::race::stage_objects::{PROBE_BYTES, course_present};
    for n in 0..COURSES {
        set_course_number(n, race, scene)?;
        set_cdat_number(course, 0);
        shared.switches.get().course_present[n as usize].set(0);
        let present = course_present(paths(course).len() as u32, |file| {
            let path = paths(course)[file as usize];
            debug_assert_eq!(PROBE_BYTES, 4);
            let bytes_read = read_whole_file_cells(
                &crate::files::cstring(path, "a course file's path")?,
                std::slice::from_ref(&scene.drawing.scene.get().work_dword),
            )?;
            Ok::<u32, Refused>(bytes_read)
        })?;
        if present {
            shared.switches.get().course_present[n as usize].set(1);
        }
    }
    Ok(())
}
