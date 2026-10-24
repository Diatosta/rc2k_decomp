//! The stage's unloading (`sub_444940`, under `sub_444AC7`, IDA 2026-10-17): its lists
//! given back. Each `race::free_*` here: the list's total taken off the course's byte
//! count (`dword_5C6F90`) but for the fonts', the list freed (`memory::free_list`), its
//! pointers 0.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::memory::{MemoryList, own_heap};

pub const COURSE_BYTES_RVA: usize = rva(0x5C6F90); // dword_5C6F90

/// One of the freeing functions: its list, whether it counts, the pointers it clears.
pub struct Freeing {
    pub list: usize,
    pub counted: bool,
    pub cleared: &'static [usize],
}

/// `race::free_course_memory` (0x41F924).
pub const COURSE: Freeing = Freeing {
    list: rva(0x5C6D50), // g_courseMemoryList
    counted: true,
    cleared: &[rva(0x5C6F94), rva(0x5C6FA0)], // g_courseMemory, g_courseInfo
};
/// `race::free_course_list_2` (0x41F94D).
pub const COURSE_2: Freeing = Freeing {
    list: rva(0x5C6D90), // stru_5C6D90
    counted: true,
    cleared: &[rva(0x5C6F98), rva(0x5C6F9C)], // dword_5C6F98, dword_5C6F9C
};
/// `race::free_point_records` (0x41F992).
pub const POINT_RECORDS: Freeing = Freeing {
    list: rva(0x5C6E50), // stru_5C6E50
    counted: true,
    cleared: &[rva(0x5C6FB8)], // g_pointRecords
};
/// `race::free_object_points` (0x41F89B).
pub const OBJECT_POINTS: Freeing = Freeing {
    list: rva(0x5C6DD0), // stru_5C6DD0
    counted: true,
    cleared: &[rva(0x5C6FA4), rva(0x5C6FA8)], // g_objectPoints, g_faceMemory
};
/// `race::free_mud` (0x449ECA).
pub const MUD: Freeing = Freeing {
    list: rva(0x5C6F50), // stru_5C6F50
    counted: true,
    cleared: &[rva(0x623D50)], // g_mud
};
/// `race::free_game_fonts` (0x41B2BC; `pusha` / `popa`, not counted).
pub const FONTS: Freeing = Freeing {
    list: rva(0x5BE898), // g_gameFontMemory
    counted: false,
    cleared: &[rva(0x51E2DC)], // g_gameFontGlyphs
};

fn list(f: &Freeing) -> *mut MemoryList {
    game_addr(f.list) as *mut MemoryList
}

/// The freeing: the count down by the list's total (`counted`), `free_list`, the
/// pointers 0. The original leaves eax the total, ebx the list and carry clear
/// (`counted`); no caller reads them (taint at every site of `render3d_tidy`,
/// `objects_tidy` and in `tidy_stage`: 0 reads). Refuses, before anything, a list with a
/// block while the game's own heap is in use (`free_list` refuses it).
pub unsafe fn free(f: &Freeing) -> Result<(), Refused> {
    unsafe {
        let l = list(f);
        if (*l).head != 0 && own_heap() == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)").into());
        }
        if f.counted {
            let total = (*l).total;
            game_write(
                COURSE_BYTES_RVA,
                game_read::<u32>(COURSE_BYTES_RVA).wrapping_sub(total),
            );
        }
        crate::memory::free_list::free_list(l)?;
        for &at in f.cleared {
            game_write(at, 0u32);
        }
    }
    Ok(())
}

/// `race::render3d_tidy` (0x4433A7): `race::free_point_records`.
pub unsafe fn render3d_tidy() -> Result<(), Refused> {
    unsafe { free(&POINT_RECORDS) }
}

/// `race::objects_tidy` (0x44346B): `race::free_object_points`.
pub unsafe fn objects_tidy() -> Result<(), Refused> {
    unsafe { free(&OBJECT_POINTS) }
}
