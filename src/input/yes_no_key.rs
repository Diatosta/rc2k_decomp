//! `input::yes_no_key` (0x407D71): says whether a key answers a yes/no question.
//!
//! In:  eax = the key, if `dword_4F7FD8` is 1; otherwise it takes the pending key itself
//! Out: eax = 1 for yes, 0xFFFFFFFF for no, 2 for the third answer, 0 for anything else or
//!      no key; everything else preserved
//! The key becomes a character through `key_to_char` (with the layout `dword_4F07F8` put in
//! place for the call when `byte_4F07D2` is 2), is made lower case with the table at
//! `byte_4F0BC8`, and is compared with the first character of strings 0xC3, 0xC4 and 0xC5 in
//! turn. Each string is built at the start of the shared text buffer without a NUL, so an
//! empty one is "compared" through whatever was there.
//! Two callers: `yes_no_for_key` and `sub_445069`.
//! Refuses a missing keyboard layout (before the key is taken) and what `key_to_char` does.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::input::key_to_char::{
    LAYOUT_A_RVA, LAYOUT_MODE_RVA, LAYOUT_RVA, has_layout, key_to_char,
};
use crate::input::take_key::take_key;
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{
    Region, Rust, at, capture, check_state, dispatch_keyed_state, restore,
};
use crate::strings::append_string_by_id::push_string;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::yes_no_key", rva(0x407D71), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const KEY_GIVEN_RVA: usize = rva(0x4F7FD8); // dword_4F7FD8: 1 = the key comes in eax
const LOWER_CASE_RVA: usize = rva(0x4F0BC8); // byte_4F0BC8: 256 characters
/// (string id, answer)
use rc2k_core::input::keyboard::ANSWERS;
pub const YES: u32 = 1;
pub const NO: u32 = u32::MAX;

/// What the two yes/no functions may write: `key_to_char`'s state and the start of the text
/// buffer.
pub fn yes_no_regions() -> Vec<Region> {
    [
        &crate::input::key_to_char::regions()[..],
        &[(text() as u32, 0x100)],
    ]
    .concat()
}

// ---- Rust version ----

/// The answer a key gives. `eax` is the register as the original has it: the key in its low
/// byte, and 0 as a whole means no key.
pub unsafe fn answer_for_key(eax: u32) -> Result<u32, BadInput> {
    if eax == 0 {
        return Ok(0);
    }
    unsafe {
        let own_layout = game_read::<u8>(LAYOUT_MODE_RVA) == 2;
        let layout: u32 = game_read(LAYOUT_RVA);
        if own_layout {
            game_write(LAYOUT_RVA, game_addr(LAYOUT_A_RVA) as u32);
        }
        let character = key_to_char(eax);
        if own_layout {
            game_write(LAYOUT_RVA, layout);
        }
        let lower = *game_ptr::<u8>(LOWER_CASE_RVA).add(character? as u8 as usize);
        // Converted: `rc2k_core::input::keyboard::answer`; each string built at the text
        // buffer's start without a NUL, as the original does (their lengths checked first).
        let strings = ANSWERS.map(|(id, _)| {
            let mut string = Vec::new();
            push_string(&mut string, id);
            string
        });
        if strings.iter().any(|s| s.len() > TEXT_LEN) {
            return Err(BadInput("a string longer than the text buffer"));
        }
        Ok(rc2k_core::input::keyboard::answer(lower, |id| {
            let n = ANSWERS.iter().position(|&(i, _)| i == id).unwrap_or(0);
            std::ptr::copy_nonoverlapping(strings[n].as_ptr(), text(), strings[n].len());
            *text()
        }))
    }
}

pub unsafe fn yes_no_key(given: u32) -> Result<u32, BadInput> {
    if !has_layout() {
        return Err(BadInput("no keyboard layout"));
    }
    unsafe {
        let eax = if game_read::<u32>(KEY_GIVEN_RVA) == 1 {
            given
        } else {
            take_key() as u32
        };
        answer_for_key(eax)
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { yes_no_key(r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &yes_no_regions(), &rust);
}

// ---- self-test: every key, given in eax ----

/// Why the yes/no functions cannot be tested, or None if they can.
pub fn yes_no_unavailable() -> Option<&'static str> {
    if !has_layout() {
        return Some("no keyboard layout");
    }
    (blob().is_null() || !ANSWERS.iter().all(|&(id, _)| is_table_id(id)))
        .then_some("the strings are not loaded")
}

/// The test of both entries: `call_original` gets the key in eax.
pub fn check_keys(t: &mut SelfTest, call_original: CallOriginal, rust: Rust) {
    let regions = [&yes_no_regions()[..], &[at(KEY_GIVEN_RVA)]].concat();
    let saved = capture(&regions);
    for high in [0u32, 0x1234_5600] {
        for key in 0..0x80u32 {
            let before = Regs {
                eax: high | key,
                ..Regs::SENTINEL
            };
            check_state(
                t,
                &format!("eax={:#x}", high | key),
                call_original,
                before,
                &|| restore(&regions, &saved),
                &regions,
                rust,
            );
        }
    }
    restore(&regions, &saved);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = yes_no_unavailable() {
        return t.skip(why);
    }
    // With the switch at 1, so that no pending key is taken from the player.
    let saved: u32 = unsafe { game_read(KEY_GIVEN_RVA) };
    unsafe { game_write(KEY_GIVEN_RVA, 1u32) };
    check_keys(&mut t, call_original, &rust);
    unsafe { game_write(KEY_GIVEN_RVA, saved) };
    t.finish()
}
