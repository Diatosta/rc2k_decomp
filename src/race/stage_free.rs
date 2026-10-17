//! The stage's unloading (`sub_444940`, under `sub_444AC7`, IDA 2026-10-17): its lists
//! given back. Each `race::free_*` here: the list's total taken off the course's byte
//! count (`dword_5C6F90`) but for the fonts', the list freed (`memory::free_list`), its
//! pointers 0.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::memory::{HELD_RVA, MOST_HELD_RVA, MemoryList, OWN_HEAP_RVA, TURNOVER_RVA};
use crate::service::harness::{Region, at};

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
/// pointers 0. Leaves eax the total and ebx the list (`counted`; else every register
/// kept). Refuses, before anything, a list with a block while the game's own heap is in
/// use (`free_list` refuses it).
pub unsafe fn free(entry: &Regs, f: &Freeing) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let l = list(f);
        if (*l).head != 0 && game_read::<u32>(OWN_HEAP_RVA) == 1 {
            return Err(BadInput("the game's own heap is in use (altmemenable)").into());
        }
        if f.counted {
            let total = (*l).total;
            game_write(
                COURSE_BYTES_RVA,
                game_read::<u32>(COURSE_BYTES_RVA).wrapping_sub(total),
            );
            g.eax = total;
            g.ebx = l as u32;
        }
        crate::memory::free_list::free_list(l)?;
        for &at in f.cleared {
            game_write(at, 0u32);
        }
    }
    Ok(g)
}

/// Whether the list holds no block (the self-test then can give it blocks of its own).
pub fn empty(f: &Freeing) -> bool {
    unsafe { (*list(f)).head == 0 }
}

/// What a freeing writes, with the memory counters.
pub fn regions(f: &Freeing) -> Vec<Region> {
    let mut r = vec![
        (game_addr(f.list) as u32, std::mem::size_of::<MemoryList>()),
        at(COURSE_BYTES_RVA),
        at(HELD_RVA),
        at(MOST_HELD_RVA),
        at(TURNOVER_RVA),
    ];
    r.extend(f.cleared.iter().map(|&a| at(a)));
    r
}

/// The self-test of a freeing: with the list empty in the game, blocks of the test's
/// (0x100, 0x2000, 0x40 bytes) put in it and freed by the original, then again by the
/// Rust version; the list, the counters, the pointers and the registers compared.
pub fn test_free(
    mut t: crate::framework::SelfTest,
    f: &Freeing,
    call_original: crate::screens::harness::CallOriginal,
) -> (usize, usize) {
    use crate::service::harness::{capture, restore};
    if !empty(f) {
        return t.skip("the list holds blocks");
    }
    let vars = regions(f);
    let saved = capture(&vars);
    // Each run starts from the same counters: `allocate` adds to the turnover (which
    // freeing never takes off), so without this the second run's turnover is the higher
    // (the first self-test run, 2026-10-17, failed so on bytes 28..31).
    let fill = |blocks: &[u32]| unsafe {
        restore(&vars, &saved);
        for &bytes in blocks {
            let p = crate::memory::allocate::allocate(list(f), bytes)
                .map(|p| p as u32)
                .unwrap_or(0);
            for &at in f.cleared {
                game_write(at, p);
            }
        }
        game_write(COURSE_BYTES_RVA, 0x0012_3456u32);
    };
    for blocks in [&[][..], &[0x100][..], &[0x100, 0x2000, 0x40][..]] {
        fill(blocks);
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = (
            capture(&vars),
            [ro.eax, ro.ebx, ro.ecx, ro.edx, ro.esi, ro.edi, ro.ebp],
        );
        fill(blocks);
        let mine = unsafe { free(&before, f) };
        let ours = (
            capture(&vars),
            mine.as_ref()
                .map(|g| [g.eax, g.ebx, g.ecx, g.edx, g.esi, g.edi, g.ebp])
                .unwrap_or_default(),
        );
        t.check(mine.is_ok() && theirs == ours, || {
            format!(
                "{blocks:x?}: original {theirs:x?}; rust {:?} {ours:x?}",
                mine.err()
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
