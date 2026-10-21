//! `menu::next_record_kind` (0x470D08): the kind (first dword) of the record after the
//! current one (`g_recordTable`, section `section`, index `index` + 1), kept in
//! `dword_7559EC`.
//!
//! In:  nothing
//! Out: eax the kind; the rest kept
//!
//! Callers: `saves::load_savegame`, the event drivers. `records::find_record`'s Rust
//! version. Refuses a section or index past 63 or a section the table does not hold,
//! before anything. Self-test: the original and the Rust version on the game's records,
//! sections and indexes by lot; put back.

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_record_kind", rva(0x470D08), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SECTION_RVA: usize = rva(0x75595C); // section
const INDEX_RVA: usize = rva(0x755958); // index
const RECORDS_RVA: usize = rva(0x735576); // g_recordTable
const KIND_RVA: usize = rva(0x7559EC); // dword_7559EC

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn next_record_kind(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let section: u32 = unsafe { game_read(SECTION_RVA) };
    let index: u32 = unsafe { game_read(INDEX_RVA) };
    let table = unsafe { Raw::new(game_addr(RECORDS_RVA) as *const u8) };
    // Converted: `rc2k_core::menu::record_kinds::next_record_kind`.
    g.eax = rc2k_core::menu::record_kinds::next_record_kind(&table, section, index)?;
    unsafe { game_write(KIND_RVA, g.eax) };
    Ok(g)
}

fn regions() -> Vec<Region> {
    vec![at(KIND_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { next_record_kind(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([SECTION_RVA, INDEX_RVA].map(at));
    let saved = capture(&vars);
    for (section, index) in [(0u32, 0u32), (1, 2), (2, 0), (3, 5), (0, 9)] {
        let table = game_addr(RECORDS_RVA) as u32;
        if !crate::records::find_record::has_section(table, section, 600) {
            continue;
        }
        check_state(
            &mut t,
            &format!("section {section}, index {index}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SECTION_RVA, section);
                game_write(INDEX_RVA, index);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
