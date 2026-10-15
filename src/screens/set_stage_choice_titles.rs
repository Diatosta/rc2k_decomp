//! `screens::set_stage_choice_titles` (0x469393): the stage choice's two titles and its
//! heading text, for the stage chosen.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The menu at 0x748620 (the stage choice's): item 0's string (+0x14) = the group's title
//! (`dword_758718[group]`); then the table's title (its +0x38) is stored at
//! `[0x748620] + 4 + 0x5C + 0x14`: the original takes the menu's first dword (a pointer
//! to its labels, `stru_747F24`) where item 0 took the menu's address, so "item 1" lands in
//! the labels, at 0x747F98 (section 16); the heading "<string 0x482> n/6: name" built in
//! `byte_A8BD54` (`stage_panel::build_second_heading`). In two entry lists. Refuses a choice
//! without its tables (the original reads through null), and what the heading refuses.
//! Self-test: the game's tables and strings, the choice's three settings over their
//! ranges, the menu, the labels and the heading compared, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::stage_choice::{GROUP_RVA, NUMBER_RVA, SUB_RVA};
use crate::screens::draw_stage_records::selection;
use crate::screens::draw_time_trial_stage_info::GROUP_TITLES_RVA;
use crate::screens::harness::Left;
use crate::screens::stage_panel::{SECOND_HEADING_RVA, build_second_heading, dword, table_dword};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::set_stage_choice_titles",
    rva(0x469393),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MENU_RVA: usize = rva(0x748620); // off_748620: the stage choice's menu
const HEADING_LEN: usize = 0x100;

/// Where item 0's string goes, and where the original puts "item 1"'s.
unsafe fn places() -> (u32, u32) {
    let menu = game_addr(MENU_RVA) as u32;
    unsafe {
        (
            menu + MENU_HEAD_LEN + 0x14,
            dword(menu).wrapping_add(MENU_HEAD_LEN + ITEM_LEN + 0x14),
        )
    }
}

pub unsafe fn set_stage_choice_titles() -> Result<(), Refused> {
    unsafe {
        let s = selection().ok_or(BadInput(
            "no stage tables for the choice: the original reads through null",
        ))?;
        let (item_0, item_1) = places();
        if item_1 < 0x1_0000 {
            return Err(BadInput("the menu's first dword is no pointer").into());
        }
        build_second_heading(s.record)?;
        (item_0 as *mut u32).write_unaligned(table_dword(GROUP_TITLES_RVA, s.group));
        (item_1 as *mut u32).write_unaligned(dword(s.table.wrapping_add(0x38)));
    }
    Ok(())
}

fn regions() -> Vec<Region> {
    let (item_0, item_1) = unsafe { places() };
    vec![
        (item_0, 4),
        (item_1, 4),
        (game_addr(SECOND_HEADING_RVA) as u32, HEADING_LEN),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_stage_choice_titles() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    if unsafe { selection() }.is_none() {
        return t.skip("no stage tables for the choice");
    }
    let regions = regions();
    let mut all = regions.clone();
    all.extend([at(GROUP_RVA), at(SUB_RVA), at(NUMBER_RVA)]);
    let saved = capture(&all);
    for group in 0..2u32 {
        for sub in [0u32, 3, 5] {
            for number in [1u32, 6] {
                check_state(
                    &mut t,
                    &format!("group {group}, rally {sub}, stage {number}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        restore(&all, &saved);
                        std::ptr::write_bytes(
                            game_addr(SECOND_HEADING_RVA) as *mut u8,
                            0xAA,
                            HEADING_LEN,
                        );
                        game_write(GROUP_RVA, group);
                        game_write(SUB_RVA, sub);
                        game_write(NUMBER_RVA, number);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
