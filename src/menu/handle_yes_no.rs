//! `menu::handle_yes_no` (0x470D2E): on a screen that asks a yes/no question, lets a key
//! answer it.
//!
//! Only while `dword_7554CC` is 1. A key that `input::yes_no_for_key` takes for yes stores 1
//! in `dword_75524C`, one for no stores 0; either also sets `dword_758806` to 1 and leaves
//! the screen with `menu::go_back`. One caller, `menu::handle_keys`.
//! Refuses what the yes/no functions refuse.

use crate::errors::BadInput;
use crate::input::yes_no_for_key::yes_no_for_key;
use crate::input::yes_no_key::{NO, YES};
use crate::menu::leaving::{question, with_question};

pub fn handle_yes_no(eax: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::leaving::Question::answer_key`.
    // A gamepad's button (not the original's), taken with the key it sent. Only Back answers:
    // this screen does not leave on Escape. Enter is a key like the keyboard's, and chooses
    // the item that is lit, yes or no.
    let pad = crate::input::menu_pad::take_answer();
    if !question().asks() {
        return Ok(());
    }
    let answer = match pad {
        Some(NO) => NO,
        _ => yes_no_for_key(eax)?,
    };
    let said = match answer {
        YES => Some(true),
        NO => Some(false),
        _ => None,
    };
    with_question(|q, l| q.answer_key(l, said));
    Ok(())
}
