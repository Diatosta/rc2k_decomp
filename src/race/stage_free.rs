//! The stage's unloading (`race::tidy_stage`, under `race::tidy_everything`): its lists are given
//! back. Each freeing here takes the list's total off the course's byte count
//! (`course_bytes_total`, except for the fonts' list), frees the list (`memory::free_list`) and
//! sets its pointers to 0.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;

/// One of the freeing functions: its list (found when it runs), whether it counts in
/// `course_bytes_total`, and the pointers it clears.
pub struct Freeing {
    pub list: fn(&Race) -> &crate::memory::ListCells,
    pub counted: bool,
    /// Each sets one pointer to 0.
    pub cleared: &'static [fn(&Race)],
    /// What Rust owns on the list, given back with it.
    pub given_back: fn(),
}

/// Frees the course memory.
pub const COURSE: Freeing = Freeing {
    // course_memory_list
    list: |race| &race.course.lists.get().course_memory_list,
    counted: true,
    // course_info
    cleared: &[|race| race.course.lists.get().course_info.set(0)],
    given_back: || {
        crate::race::scene_points::give_back();
        crate::race::course_places::give_back();
    },
};
/// Frees the second course list.
pub const COURSE_2: Freeing = Freeing {
    // CourseLists.second_course_list[0]
    list: |race| &race.course.lists.get().second_course_list,
    counted: true,
    // `second_course_list_a`, `second_course_list_b`
    cleared: &[
        |race| {
            race.course.lists.get().second_course_list_a.set(0);
        },
        |race| {
            race.course.lists.get().second_course_list_b.set(0);
        },
    ],
    given_back: || {},
};
/// Frees the point records.
pub const POINT_RECORDS: Freeing = Freeing {
    // CourseLists.point_records_list[0]
    list: |race| &race.course.lists.get().point_records_list,
    counted: true,
    // point_records
    cleared: &[|race| race.course.lists.get().point_records.set(0)],
    given_back: crate::scene::point_records::give_back,
};
/// Frees the object points.
pub const OBJECT_POINTS: Freeing = Freeing {
    // CourseLists.object_points_list[0]
    list: |race| &race.course.lists.get().object_points_list,
    counted: true,
    // object_points, face_memory
    cleared: &[
        |race| race.course.lists.get().object_points.set(0),
        |race| race.course.lists.get().face_memory.set(0),
    ],
    given_back: || {
        crate::scene::object_points::give_back();
        crate::scene::object_faces::give_back();
    },
};
/// Frees the mud.
pub const MUD: Freeing = Freeing {
    // CourseLists.mud_list[0]
    list: |race| &race.course.lists.get().mud_list,
    counted: true,
    // mud
    cleared: &[|race| race.shared.vars.get().mud.set(0)],
    given_back: || crate::race::mud::MUD.give_back(),
};

/// `race::free_game_fonts`.
pub fn free_game_fonts(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let l = &gfx.fonts.game_fonts_memory.memory;
    let list = l.get();
    if list.head != 0 {
        return Err(BadInput("a memory list holding a block").into());
    }
    crate::memory::free_list::free_list(l)?;
    // game_font_glyphs
    gfx.fonts.font_block.game_fonts[0].update(|f| f.glyph_table = 0);
    Ok(())
}

/// Frees a list: lowers the count by the list's total (`counted`), runs `free_list` and sets the
/// pointers to 0.
/// Refuses, before anything, a list with a block (`free_list` refuses it).
pub fn free(race: &Race, f: &Freeing) -> Result<(), Refused> {
    let course = &race.course;
    let l = (f.list)(race);
    let list = l.get();
    if list.head != 0 {
        return Err(BadInput("a memory list holding a block").into());
    }
    if f.counted {
        // `course_bytes_total`
        course.lists.get().course_bytes_total.set(
            course
                .lists
                .get()
                .course_bytes_total
                .get()
                .wrapping_sub(list.total),
        );
    }
    crate::memory::free_list::free_list(l)?;
    for clear in f.cleared {
        clear(race);
    }
    (f.given_back)();
    Ok(())
}

/// `race::render3d_tidy`: frees the point records (`POINT_RECORDS`).
pub fn render3d_tidy(race: &Race) -> Result<(), Refused> {
    free(race, &POINT_RECORDS)
}

/// `race::objects_tidy`: frees the object points (`OBJECT_POINTS`).
pub fn objects_tidy(race: &Race) -> Result<(), Refused> {
    free(race, &OBJECT_POINTS)
}
