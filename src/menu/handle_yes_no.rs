//! `menu::handle_yes_no`: on a screen that asks a yes/no question, lets a key
//! answer it.
//!
//! Only while `flow.asking` is 1. A key that `input::yes_no_for_key` takes for yes stores 1 in
//! `pages.yes_no_answer`, one for no stores 0; either also sets `captions.left_by_key` to 1 and
//! leaves the screen with `menu::go_back`. Refuses what the yes/no
//! functions refuse.

use crate::errors::BadInput;
use crate::input::yes_no_for_key::yes_no_for_key;
use crate::input::yes_no_key::{NO, YES};
use crate::menu::leaving::{question, with_question};
use crate::menu::owner::Menus;

pub fn handle_yes_no(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    input: &crate::input::owner::Input,
    menus: &Menus,
    key: u32,
) -> Result<(), BadInput> {
    // A gamepad's button, taken with the key it sent. Only Back answers:
    // this screen does not leave on Escape. Enter is a key like the keyboard's, and chooses
    // the item that is lit, yes or no.
    let pad = crate::input::menu_pad::take_answer(input);
    if !question(
        &menus.screens.flow,
        &menus.tables.pages,
        &menus.tables.captions,
    )
    .asks()
    {
        return Ok(());
    }
    let answer = match pad {
        Some(NO) => NO,
        _ => yes_no_for_key(so, input, key)?,
    };
    let said = match answer {
        YES => Some(true),
        NO => Some(false),
        _ => None,
    };
    with_question(menus, |q, l| q.answer_key(l, said));
    Ok(())
}
