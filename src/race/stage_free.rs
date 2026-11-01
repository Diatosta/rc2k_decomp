//! The stage's unloading (`sub_444940`, under `sub_444AC7`, IDA 2026-10-17): its lists
//! given back. Each `race::free_*` here: the list's total taken off the course's byte
//! count (`dword_5C6F90`) but for the fonts', the list freed (`memory::free_list`), its
//! pointers 0.

use crate::errors::{BadInput, Refused};
use crate::memory::{MemoryList, own_heap};
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::vars_store::RACE_VARS;

/// One of the freeing functions: its list (where it is, found when it runs: the statics
/// deref at run time), whether it counts, the pointers it clears.
pub struct Freeing {
    pub list: fn() -> crate::pointers::GamePtr,
    pub counted: bool,
    /// Each sets one of the pointers to 0.
    pub cleared: &'static [fn()],
    /// What Rust owns on the list, given back with it.
    pub given_back: fn(),
}

/// `race::free_course_memory` (0x41F924).
pub const COURSE: Freeing = Freeing {
    // g_courseMemoryList
    list: || crate::pointers::GamePtr::to(&COURSE_LISTS.course_memory_list),
    counted: true,
    // g_courseMemory, g_courseInfo
    cleared: &[
        || COURSE_LISTS.course_memory.set(0),
        || COURSE_LISTS.course_info.set(0),
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
    list: || crate::pointers::GamePtr::to(&COURSE_LISTS.course_list_2),
    counted: true,
    // dword_5C6F98, dword_5C6F9C
    cleared: &[
        || COURSE_LISTS.course_list_2_a.set(0),
        || COURSE_LISTS.course_list_2_b.set(0),
    ],
    given_back: || {},
};
/// `race::free_point_records` (0x41F992).
pub const POINT_RECORDS: Freeing = Freeing {
    // stru_5C6E50
    list: || crate::pointers::GamePtr::to(&COURSE_LISTS.point_records_list),
    counted: true,
    // g_pointRecords
    cleared: &[|| COURSE_LISTS.point_records.set(0)],
    given_back: crate::scene::point_records::give_back,
};
/// `race::free_object_points` (0x41F89B).
pub const OBJECT_POINTS: Freeing = Freeing {
    // stru_5C6DD0
    list: || crate::pointers::GamePtr::to(&COURSE_LISTS.object_points_list),
    counted: true,
    // g_objectPoints, g_faceMemory
    cleared: &[
        || COURSE_LISTS.object_points.set(0),
        || COURSE_LISTS.face_memory.set(0),
    ],
    given_back: || {
        crate::scene::object_points::give_back();
        crate::scene::object_faces::give_back();
    },
};
/// `race::free_mud` (0x449ECA).
pub const MUD: Freeing = Freeing {
    // stru_5C6F50
    list: || crate::pointers::GamePtr::to(&COURSE_LISTS.mud_list),
    counted: true,
    // g_mud
    cleared: &[|| RACE_VARS.mud.set(0)],
    given_back: || crate::race::mud::MUD.give_back(),
};
/// `race::free_game_fonts` (0x41B2BC; `pusha` / `popa`, not counted).
pub const FONTS: Freeing = Freeing {
    // g_gameFontMemory
    list: || crate::pointers::GamePtr::to(&crate::gfx::build_game_fonts::GAME_FONTS.memory),
    counted: false,
    // g_gameFontGlyphs
    cleared: &[|| crate::gfx::fonts_store::FONTS.game_fonts[0].update(|f| f.glyph_table = 0)],
    given_back: || {},
};

/// The freeing: the count down by the list's total (`counted`), `free_list`, the
/// pointers 0. The original leaves eax the total, ebx the list and carry clear
/// (`counted`); no caller reads them (taint at every site of `render3d_tidy`,
/// `objects_tidy` and in `tidy_stage`: 0 reads). Refuses, before anything, a list with a
/// block while the game's own heap is in use (`free_list` refuses it).
pub fn free(f: &Freeing) -> Result<(), Refused> {
    let l = (f.list)().unbounded(); // As in `free_list`.
    let list = l.read::<MemoryList>("a memory list")?;
    if list.head != 0 && own_heap() == 1 {
        return Err(BadInput("the game's own heap is in use (altmemenable)").into());
    }
    if f.counted {
        // dword_5C6F90
        COURSE_LISTS.course_bytes_total.set(
            COURSE_LISTS
                .course_bytes_total
                .bits()
                .wrapping_sub(list.total),
        );
    }
    crate::memory::free_list::free_list(l)?;
    for clear in f.cleared {
        clear();
    }
    (f.given_back)();
    Ok(())
}

/// `race::render3d_tidy` (0x4433A7): `race::free_point_records`.
pub fn render3d_tidy() -> Result<(), Refused> {
    free(&POINT_RECORDS)
}

/// `race::objects_tidy` (0x44346B): `race::free_object_points`.
pub fn objects_tidy() -> Result<(), Refused> {
    free(&OBJECT_POINTS)
}
