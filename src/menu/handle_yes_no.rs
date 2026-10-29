//! `menu::handle_yes_no` (0x470D2E): on a screen that asks a yes/no question, lets a key
//! answer it.
//!
//! In:  eax = the key
//! Out: every register preserved
//! Only while `dword_7554CC` is 1. A key that `input::yes_no_for_key` takes for yes stores 1
//! in `dword_75524C`, one for no stores 0; either also sets `dword_758806` to 1 and leaves
//! the screen with `menu::go_back`. One caller, `menu::handle_keys`.
//! Refuses what the yes/no functions refuse.

use crate::errors::BadInput;
use crate::input::yes_no_for_key::yes_no_for_key;
use crate::input::yes_no_key::{NO, YES};
use crate::menu::leaving::{question, with_question};

// ---- Rust version ----

pub fn handle_yes_no(eax: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::leaving::Question::answer_key`.
    if !question().asks() {
        return Ok(());
    }
    let said = match { yes_no_for_key(eax) }? {
        YES => Some(true),
        NO => Some(false),
        _ => None,
    };
    with_question(|q, l| q.answer_key(l, said));
    Ok(())
}
