//! The game's leaving switches and yes/no question as `rc2k_core::menu::leaving`'s
//! `Leaving` and `Question`: read, changed, written back whole.
use crate::menu::owner::menus;
use rc2k_core::menu::leaving::{Leaving, Question};

pub fn leaving() -> Leaving {
    let screens = &menus().screens;
    let flags = &screens.loop_flags;
    Leaving {
        event_left: flags.event_left.is_set(),
        leave_screen: flags.leave_screen.is_set(),
        left_with_carry: flags.left_with_carry.is_set(),
        ask_leave: flags.ask_leave.is_set(),
        scratch_0c: screens.draw_work.scratch_0c.get(),
    }
}

pub fn put_leaving(l: &Leaving) {
    let screens = &menus().screens;
    let flags = &screens.loop_flags;
    flags.event_left.set(l.event_left);
    flags.leave_screen.set(l.leave_screen);
    flags.left_with_carry.set(l.left_with_carry);
    flags.ask_leave.set(l.ask_leave);
    screens.draw_work.scratch_0c.set(l.scratch_0c);
}

pub fn question() -> Question {
    let menus = menus();
    Question {
        asking: menus.screens.flow.asking.is_set(),
        answer: menus.tables.pages.yes_no_answer.get(),
        answered: menus.tables.captions.left_by_key.is_set(),
    }
}

pub fn put_question(q: &Question) {
    let menus = menus();
    menus.screens.flow.asking.set(q.asking);
    menus.tables.pages.yes_no_answer.set(q.answer);
    menus.tables.captions.left_by_key.set(q.answered);
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
