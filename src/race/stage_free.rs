//! The stage's unloading (`race::tidy_stage`, under `race::tidy_everything`, IDA 2026-10-17): its
//! lists given back. Each `race::free_*` here: the list's total taken off the course's byte
//! count (`dword_5C6F90`) but for the fonts', the list freed (`memory::free_list`), its
//! pointers 0.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::memory::MemoryList;
use crate::race::owner::Race;

/// One of the freeing functions: its list (where it is, found when it runs: the statics
/// deref at run time), whether it counts, the pointers it clears.
pub struct Freeing {
    pub list: fn(&Race) -> &dyn crate::pointers::Store,
    pub counted: bool,
    /// Each sets one of the pointers to 0.
    pub cleared: &'static [fn(&Race)],
    /// What Rust owns on the list, given back with it.
    pub given_back: fn(),
}

/// `race::free_course_memory` (0x41F924).
pub const COURSE: Freeing = Freeing {
    // g_courseMemoryList
    list: |race| &race.course.lists.course_memory_list,
    counted: true,
    // g_courseMemory, g_courseInfo
    cleared: &[
        |race| race.course.lists.course_memory.set(0),
        |race| race.course.lists.course_info.set(0),
    ],
    given_back: || {
        crate::race::scene_points::give_back();
        crate::race::course_places::give_back();
        crate::race::level_init::give_back_course_memory();
        for name in [
            "course records",
            "course faces",
            "old scene points",
            "course info",
        ] {
            crate::pointers::unwatch(name);
        }
    },
};
/// `race::free_course_list_2` (0x41F94D).
pub const COURSE_2: Freeing = Freeing {
    // stru_5C6D90
    list: |race| &race.course.lists.second_course_list,
    counted: true,
    // dword_5C6F98, dword_5C6F9C
    cleared: &[
        |race| {
            race.course.lists.second_course_list_a.set(0);
        },
        |race| {
            race.course.lists.second_course_list_b.set(0);
        },
    ],
    given_back: || {},
};
/// `race::free_point_records` (0x41F992).
pub const POINT_RECORDS: Freeing = Freeing {
    // stru_5C6E50
    list: |race| &race.course.lists.point_records_list,
    counted: true,
    // g_pointRecords
    cleared: &[|race| race.course.lists.point_records.set(0)],
    given_back: crate::scene::point_records::give_back,
};
/// `race::free_object_points` (0x41F89B).
pub const OBJECT_POINTS: Freeing = Freeing {
    // stru_5C6DD0
    list: |race| &race.course.lists.object_points_list,
    counted: true,
    // g_objectPoints, g_faceMemory
    cleared: &[
        |race| race.course.lists.object_points.set(0),
        |race| race.course.lists.face_memory.set(0),
    ],
    given_back: || {
        crate::scene::object_points::give_back();
        crate::scene::object_faces::give_back();
    },
};
/// `race::free_mud` (0x449ECA).
pub const MUD: Freeing = Freeing {
    // stru_5C6F50
    list: |race| &race.course.lists.mud_list,
    counted: true,
    // g_mud
    cleared: &[|race| race.shared.vars.mud.set(0)],
    given_back: || crate::race::mud::MUD.give_back(),
};
/// `race::free_game_fonts` (0x41B2BC; `pusha` / `popa`, not counted).
pub const FONTS: Freeing = Freeing {
    // g_gameFontMemory
    list: |_| &gfx().fonts.game_fonts_memory.memory,
    counted: false,
    // g_gameFontGlyphs
    cleared: &[|_| {
        gfx().fonts.font_block.game_fonts[0].update(|f| f.glyph_table = 0);
    }],
    given_back: || {},
};

/// The freeing: the count down by the list's total (`counted`), `free_list`, the pointers 0.
/// Refuses, before anything, a list with a block (`free_list` refuses it).
pub fn free(race: &Race, f: &Freeing) -> Result<(), Refused> {
    let course = &race.course;
    let l = (f.list)(race);
    let list = crate::pointers::load_value::<MemoryList>(l, 0)?;
    if list.head != 0 {
        return Err(BadInput("a memory list holding a block").into());
    }
    if f.counted {
        // dword_5C6F90
        course.lists.course_bytes_total.set(
            course
                .lists
                .course_bytes_total
                .bits()
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

/// `race::render3d_tidy` (0x4433A7): `race::free_point_records`.
pub fn render3d_tidy(race: &Race) -> Result<(), Refused> {
    free(race, &POINT_RECORDS)
}

/// `race::objects_tidy` (0x44346B): `race::free_object_points`.
pub fn objects_tidy(race: &Race) -> Result<(), Refused> {
    free(race, &OBJECT_POINTS)
}
