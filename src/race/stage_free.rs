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
    pub given_back: fn(&Race, &crate::scene::scratch_store::DrawingScratch),
}

/// Frees the course memory.
pub(super) const COURSE: Freeing = Freeing {
    // course_memory_list
    list: |race| &race.course.memory().course_memory_list,
    counted: true,
    cleared: &[],
    given_back: |race, _| {
        crate::race::scene_points::give_back(&race.course.stage.course);
        crate::race::course_places::give_back(&race.course.stage.course);
    },
};
/// Frees the second course list.
pub(super) const COURSE_2: Freeing = Freeing {
    list: |race| &race.course.memory().second_course_list,
    counted: true,
    // `second_course_list_a`, `second_course_list_b`
    cleared: &[
        |race| {
            race.course.lists.borrow_mut().second_course_list_a = 0;
        },
        |race| {
            race.course.lists.borrow_mut().second_course_list_b = 0;
        },
    ],
    given_back: |_, _| {},
};
/// Frees the point records.
pub(super) const POINT_RECORDS: Freeing = Freeing {
    list: |race| &race.course.memory().point_records_list,
    counted: true,
    // point_records
    cleared: &[|race| race.course.lists.borrow_mut().point_records = 0],
    given_back: |_, scratch| crate::scene::point_records::give_back(scratch),
};
/// Frees the object points.
pub(super) const OBJECT_POINTS: Freeing = Freeing {
    list: |race| &race.course.memory().object_points_list,
    counted: true,
    // object_points, face_memory
    cleared: &[
        |race| race.course.lists.borrow_mut().object_points = 0,
        |race| race.course.lists.borrow_mut().face_memory = 0,
    ],
    given_back: |race, _| {
        race.course.stage.object_points.give_back();
        race.course.stage.object_faces.give_back();
    },
};
/// Frees the mud.
pub(super) const MUD: Freeing = Freeing {
    list: |race| &race.course.memory().mud_list,
    counted: true,
    // mud
    cleared: &[|race| race.shared.vars.get().mud.set(0)],
    given_back: |race, _| race.course.stage.mud.give_back(),
};

/// `race::free_game_fonts`.
pub(super) fn free_game_fonts(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let l = &gfx.fonts.game_fonts_memory.memory;
    let list = l.get();
    if list.head != 0 {
        return Err(BadInput("a memory list holding a block").into());
    }
    crate::gfx::glyph_store::free_list(&gfx.fonts, l)?;
    // game_font_glyphs
    gfx.fonts.font_block.game_fonts[0].update(|f| f.glyph_table = 0);
    Ok(())
}

/// Frees a list: lowers the count by the list's total (`counted`), runs `free_list` and sets the
/// pointers to 0.
/// Refuses, before anything, a list with a block (`free_list` refuses it).
pub fn free(
    race: &Race,
    scratch: &crate::scene::scratch_store::DrawingScratch,
    f: &Freeing,
) -> Result<(), Refused> {
    let course = &race.course;
    let l = (f.list)(race);
    let list = l.get();
    if list.head != 0 {
        return Err(BadInput("a memory list holding a block").into());
    }
    if f.counted {
        // `course_bytes_total`
        {
            let mut lists = course.lists.borrow_mut();
            lists.course_bytes_total = lists.course_bytes_total.wrapping_sub(list.total);
        }
    }
    crate::memory::free_list::free_list(l)?;
    for clear in f.cleared {
        clear(race);
    }
    (f.given_back)(race, scratch);
    Ok(())
}

/// `race::render3d_tidy`: frees the point records (`POINT_RECORDS`).
pub fn render3d_tidy(
    race: &Race,
    scratch: &crate::scene::scratch_store::DrawingScratch,
) -> Result<(), Refused> {
    free(race, scratch, &POINT_RECORDS)
}

/// `race::objects_tidy`: frees the object points (`OBJECT_POINTS`).
pub fn objects_tidy(
    race: &Race,
    scratch: &crate::scene::scratch_store::DrawingScratch,
) -> Result<(), Refused> {
    free(race, scratch, &OBJECT_POINTS)
}
