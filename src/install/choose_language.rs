//! `install::choose_language` (0x41397A): makes one of the languages of `lang.cfg` the
//! language of the paths.
//!
//! In:  eax = the language's number, from 1
//! Out: every register preserved
//! Nothing if the number is above the count of languages (`dword_51BEEA`). A language is a
//! slot of 0x80 bytes at 0x51BEEE holding two texts, each ended by a NUL (`name:directory`
//! in the file); the second becomes what `%` stands for in a path (`byte_518AA8`).
//! Callers: `install::read_languages`, `sub_412F34`, `sub_414199`.
//! Refuses number 0 (the original takes the slot before the first) and a slot whose two
//! texts do not end inside it.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::install::{LOCALE_RVA, PATH_ROOM};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::install::config;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::choose_language", rva(0x41397A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const LANGUAGE_COUNT_RVA: usize = rva(0x51BEEA); // dword_51BEEA
pub const LANGUAGES_RVA: usize = rva(0x51BEEE); // unk_51BEEE
pub const LANGUAGE_SLOT: usize = config::SLOT;
pub const LANGUAGE_SLOTS: usize = config::SLOTS;

// ---- Rust version ----

pub unsafe fn choose_language(number: u32) -> Result<(), BadInput> {
    unsafe {
        // Converted: `rc2k_core::install::config` (`language_slot`, `language_directory`).
        let Some(slot) = config::language_slot(number, game_read::<u32>(LANGUAGE_COUNT_RVA))?
        else {
            return Ok(());
        };
        let slot = std::slice::from_raw_parts(
            game_addr(LANGUAGES_RVA + slot * LANGUAGE_SLOT) as *const u8,
            LANGUAGE_SLOT,
        );
        let directory = config::language_directory(slot)?;
        let to = game_addr(LOCALE_RVA) as *mut u8;
        std::ptr::copy(directory.as_ptr(), to, directory.len());
        to.add(directory.len()).write(0);
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(LOCALE_RVA) as u32, PATH_ROOM)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { choose_language(r.eax) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: [Region; 3] = [
        regions()[0],
        at(LANGUAGE_COUNT_RVA),
        (
            game_addr(LANGUAGES_RVA) as u32,
            LANGUAGE_SLOT * LANGUAGE_SLOTS,
        ),
    ];
    let saved = capture(&all);
    for count in [0u32, 1, 3, 9] {
        for number in 1..=10u32 {
            let prepare = || unsafe {
                restore(&all, &vec![0xAA; saved.len()]);
                game_write(LANGUAGE_COUNT_RVA, count);
                for slot in 0..LANGUAGE_SLOTS {
                    let text = format!("name{slot}\0files\\loc{slot}{}\0", "x".repeat(slot * 3));
                    std::ptr::copy(
                        text.as_ptr(),
                        game_addr(LANGUAGES_RVA + slot * LANGUAGE_SLOT) as *mut u8,
                        text.len(),
                    );
                }
            };
            check_state(
                &mut t,
                &format!("count={count} number={number}"),
                call_original,
                Regs {
                    eax: number,
                    ..Regs::SENTINEL
                },
                &prepare,
                &regions(),
                &rust,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
