//! The game's leaving switches and yes/no question as `rc2k_core::menu::leaving`'s
//! `Leaving` and `Question`: read, changed, written back whole.
use crate::menu::owner::Menus;
use rc2k_core::menu::leaving::{Leaving, Question};

pub fn leaving(menus: &Menus) -> Leaving {
    let screens = &menus.screens;
    let flags = &screens.loop_flags;
    Leaving {
        event_left: flags.event_left.is_set(),
        leave_screen: flags.leave_screen.is_set(),
        left_with_carry: flags.left_with_carry.is_set(),
        ask_leave: flags.ask_leave.is_set(),
        scratch_0c: screens.draw_work.scratch_0c.get(),
    }
}

pub fn put_leaving(menus: &Menus, l: &Leaving) {
    let screens = &menus.screens;
    let flags = &screens.loop_flags;
    flags.event_left.set(l.event_left);
    flags.leave_screen.set(l.leave_screen);
    flags.left_with_carry.set(l.left_with_carry);
    flags.ask_leave.set(l.ask_leave);
    screens.draw_work.scratch_0c.set(l.scratch_0c);
}

pub fn question(menus: &Menus) -> Question {
    Question {
        asking: menus.screens.flow.asking.is_set(),
        answer: menus.tables.pages.yes_no_answer.get(),
        answered: menus.tables.captions.left_by_key.is_set(),
    }
}

pub fn put_question(menus: &Menus, q: &Question) {
    menus.screens.flow.asking.set(q.asking);
    menus.tables.pages.yes_no_answer.set(q.answer);
    menus.tables.captions.left_by_key.set(q.answered);
}

/// The switches changed by `f`.
pub fn with_leaving<R>(menus: &Menus, f: impl FnOnce(&mut Leaving) -> R) -> R {
    let mut l = leaving(menus);
    let r = f(&mut l);
    put_leaving(menus, &l);
    r
}

/// The question and the switches changed by `f`.
pub fn with_question<R>(menus: &Menus, f: impl FnOnce(&mut Question, &mut Leaving) -> R) -> R {
    let (mut q, mut l) = (question(menus), leaving(menus));
    let r = f(&mut q, &mut l);
    put_question(menus, &q);
    put_leaving(menus, &l);
    r
}
