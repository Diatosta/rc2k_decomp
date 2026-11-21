//! Finds which of the 100 courses are there: for each, sets its number in the stage's paths
//! (`race::set_course_number`, `race::set_cdat_number` with 0), its flag (`course_present[n]`) to
//! 0, and reads 4 bytes of `srclvls\levelNN\cdat00c` into `work_dword` (`files::read_whole_file`);
//! if four are read (`read_bytes`) the flag is 1; else it tries the same with
//! `files\levels\levelNN\cdat00c.sq`. Refused: what `files::read_whole_file` refuses (nothing
//! written then).

use crate::errors::Refused;
use crate::files::read_whole_file::read_whole_file_cells;
use crate::race::course_data::set_cdat_number;
use crate::race::level_init::set_course_number;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// `srclvls\level01\cdat00c`, `files\levels\level01\cdat00c.sq`.
fn paths(course: &Course) -> Result<[Vec<u8>; 2], crate::errors::BadInput> {
    let p = course.level_paths();
    Ok([
        crate::files::cstring_in(&p.cdat_src)?,
        crate::files::cstring_in(&p.cdat_files)?,
    ])
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
        shared.switches.borrow_mut().course_present[n as usize] = 0;
        let present = course_present(2, |file| {
            let path = paths(course)?
                .into_iter()
                .nth(file as usize)
                .unwrap_or_default();
            debug_assert_eq!(PROBE_BYTES, 4);
            let probe = [crate::memory::session::PlainDword::new(
                scene.draw_work().work_dword,
            )];
            let read = read_whole_file_cells(&path, &probe);
            scene.draw_work_mut().work_dword = probe[0].get();
            Ok::<u32, Refused>(read?)
        })?;
        if present {
            shared.switches.borrow_mut().course_present[n as usize] = 1;
        }
    }
    Ok(())
}
