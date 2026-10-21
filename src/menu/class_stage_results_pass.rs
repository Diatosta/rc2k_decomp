//! `menu::class_stage_results_pass` (0x4A1EE6): standings table A sorted by stage time (`menu::sort_standings_by_time`), then for each class from 1 (with `dword_75AB40` 2) or 0 up to `dword_7558F8`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table A, the class numbered (`menu::number_standings_a`, its
//! count into `dword_75597C`) and its page shown (`menu::show_class_stage_results`), stopping when the
//! leaving is answered yes (`g_yesNoAnswer` 1). `dword_755565` the class's flag.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The steps are their Rust versions; the page (inside `show_class_stage_results`)
//! through `part` (1). Ungated (the pages wait for the player). Refuses a table without its
//! end entry and a class count the loop would run past (at or below the first, or past
//! 0x100), before anything; what the steps refuse. Self-test: a copy whose page wrapper
//! call goes to a stand-in, against the Rust version on the same, on the game's tables;
//! put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::menu::class_pages;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::class_stage_results_pass",
    rva(0x4A1EE6),
    thunk,
    &ORIGINAL,
)
.private_copy(0x80)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

/// Its calls (the first in the patched bytes).
const CALLS: [usize; 3] = [0x1, 0x5C, 0x67];

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FLAG_RVA: usize = rva(0x755565); // dword_755565
const COUNT_RVA: usize = rva(0x75597C); // dword_75597C
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const MODE_RVA: usize = rva(0x75AB40); // dword_75AB40
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// A call by place: 1 the page (passed to `show_class_stage_results`), 2 the page wrapper itself (the
/// tests' stand-in).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The first class (see the header).
fn first_class() -> u32 {
    class_pages::first_stage_class(unsafe { game_read(MODE_RVA) })
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn class_stage_results_pass(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        crate::menu::sort_standings_by_time::sort_standings_by_time(entry)?;
        let first = first_class();
        // Converted: `rc2k_core::menu::class_pages` (`classes`, `has_player`).
        let classes = class_pages::classes(first, game_read(CLASSES_RVA))?;
        let table: u32 = game_read(TABLE_A_RVA);
        let all = entries(table, A_ENTRY)?;
        for class in classes {
            game_write(FLAG_RVA, 0u32);
            if !class_pages::has_player(
                all.len(),
                |n| get(all[n] + 0x1C),
                |n| get(all[n] + 4),
                class,
            ) {
                continue;
            }
            game_write(FLAG_RVA, 1u32);
            let mut g = crate::menu::number_standings_a::number_standings_a(&Regs {
                ebx: class,
                ..*entry
            })?;
            game_write(COUNT_RVA, g.ecx);
            part(2, &mut g)?;
            if game_read::<u32>(ANSWER_RVA) == 1 {
                break;
            }
        }
    }
    Ok(*entry)
}

/// In the game: the page wrapper's Rust version (its page the game's Rust version).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::menu::show_class_stage_results as page;
    if place == 2 {
        *r = unsafe { page::show_class_stage_results(r, &mut page::game_part) }?;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { class_stage_results_pass(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4A1EE6,
        0x80,
        &[0x60, 0xE8, 0xD8, 0x93, 0xFD, 0xFF],
        &CALLS,
        &[(0x4A1F7A, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let table: u32 = unsafe { game_read(TABLE_A_RVA) };
    let Ok(all) = entries(table, A_ENTRY) else {
        return t.skip("the game's table is not there");
    };
    let mut regions: Vec<Region> = vec![
        (table, all.len() * A_ENTRY as usize),
        at(FLAG_RVA),
        at(COUNT_RVA),
    ];
    regions.extend(
        (0..4u32)
            .map(|n| (player(n), 0x400))
            .filter(|&(p, _)| p != 0),
    );
    let mut vars = regions.clone();
    vars.extend([CLASSES_RVA, ANSWER_RVA, MODE_RVA, EVENT_MODE_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(CLASSES_RVA, 4u32);
                game_write(MODE_RVA, if case == 1 { 2u32 } else { 0 });
                game_write(EVENT_MODE_RVA, if case == 2 { 0x13u32 } else { 0 });
                for (k, &e) in all.iter().enumerate().take(12) {
                    put(e + 0x1C, k as u32 % 4);
                    put(
                        e + 4,
                        if k % 3 == 0 {
                            1 + (k as u32 / 3) % 4
                        } else {
                            0
                        },
                    );
                }
            },
            &|r| unsafe { class_stage_results_pass(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
