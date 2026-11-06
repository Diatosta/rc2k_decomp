//! The game's leaving switches and yes/no question as `rc2k_core::menu::leaving`'s
//! `Leaving` and `Question`: read, changed, written back whole.
use crate::menu::owner::menus;
use crate::session::Dword;
use rc2k_core::menu::leaving::{Leaving, Question};

/// The switches of `Leaving`, in its order: `dword_75555D`, `dword_755561`, `dword_755565`,
/// `dword_755569` and `dword_755A0C`.
fn leaving_cells() -> [&'static Dword; 5] {
    let screens = &menus().screens;
    [
        &screens.loop_flags.went_back,
        &screens.loop_flags.back,
        &screens.loop_flags.ended_b,
        &screens.loop_flags.ended_a,
        &screens.draw_work.work_0c,
    ]
}

/// The cells of `Question`, in its order: `dword_7554CC` (1 on a yes / no screen),
/// `g_yesNoAnswer` (1 = yes, 0 = no) and `dword_758806` (the screen was left).
fn question_cells() -> [&'static Dword; 3] {
    let menus = menus();
    [
        &menus.screens.flow.asking,
        &menus.tables.pages.yes_no_answer,
        &menus.tables.captions.screen_left,
    ]
}

fn get<const N: usize>(at: [&'static Dword; N]) -> [u32; N] {
    at.map(super::super::session::Dword::get)
}

fn put<const N: usize>(at: [&'static Dword; N], v: [u32; N]) {
    for (cell, v) in at.into_iter().zip(v) {
        cell.set(v);
    }
}

pub fn leaving() -> Leaving {
    let [leave_event, leave_screen, leave_too, ask, field_a0c] = get(leaving_cells());
    Leaving {
        leave_event,
        leave_screen,
        leave_too,
        ask,
        field_a0c,
    }
}

pub fn put_leaving(l: &Leaving) {
    put(
        leaving_cells(),
        [
            l.leave_event,
            l.leave_screen,
            l.leave_too,
            l.ask,
            l.field_a0c,
        ],
    );
}

pub fn question() -> Question {
    let [asking, answer, answered] = get(question_cells());
    Question {
        asking,
        answer,
        answered,
    }
}

pub fn put_question(q: &Question) {
    put(question_cells(), [q.asking, q.answer, q.answered]);
}

/// The switches changed by `f`.
pub fn with_leaving<R>(f: impl FnOnce(&mut Leaving) -> R) -> R {
    let mut l = leaving();
    let r = f(&mut l);
    put_leaving(&l);
    r
}

/// The question and the switches changed by `f`.
pub fn with_question<R>(f: impl FnOnce(&mut Question, &mut Leaving) -> R) -> R {
    let (mut q, mut l) = (question(), leaving());
    let r = f(&mut q, &mut l);
    put_question(&q);
    put_leaving(&l);
    r
}
