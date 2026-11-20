//! The game's leaving switches and yes/no question as `rc2k_core::menu::leaving`'s
//! `Leaving` and `Question`: read, changed, written back whole.
use crate::menu::captions_store::CaptionTables;
use crate::menu::flow_store::ScreenFlow;
use crate::menu::owner::Menus;
use crate::menu::pages_store::MenuPages;
use rc2k_core::menu::leaving::{Leaving, Question};

pub fn leaving(menus: &Menus) -> Leaving {
    let screens = &menus.screens;
    let flags = screens.loop_flags.read();
    Leaving {
        event_left: flags.event_left,
        leave_screen: flags.leave_screen,
        left_with_carry: flags.left_with_carry,
        ask_leave: flags.ask_leave,
        result: screens.draw_work.read().leave_result,
    }
}

fn put_leaving(menus: &Menus, l: &Leaving) {
    let screens = &menus.screens;
    let mut flags = screens.loop_flags.write();
    flags.event_left = l.event_left;
    flags.leave_screen = l.leave_screen;
    flags.left_with_carry = l.left_with_carry;
    flags.ask_leave = l.ask_leave;
    drop(flags);
    screens.draw_work.write().leave_result = l.result;
}

pub fn question(flow: &ScreenFlow, pages: &MenuPages, captions: &CaptionTables) -> Question {
    Question {
        asking: flow.read().asking,
        answer: pages.read().yes_no_answer,
        answered: captions.left_by_key.is_set(),
    }
}

fn put_question(flow: &ScreenFlow, pages: &MenuPages, captions: &CaptionTables, q: &Question) {
    flow.write().asking = q.asking;
    pages.write().yes_no_answer = q.answer;
    captions.left_by_key.set(q.answered);
}

/// The switches changed by `f`.
pub(super) fn with_leaving<R>(menus: &Menus, f: impl FnOnce(&mut Leaving) -> R) -> R {
    let mut l = leaving(menus);
    let r = f(&mut l);
    put_leaving(menus, &l);
    r
}

/// The question and the switches changed by `f`.
pub(super) fn with_question<R>(
    menus: &Menus,
    f: impl FnOnce(&mut Question, &mut Leaving) -> R,
) -> R {
    let tables = &menus.tables;
    let (mut q, mut l) = (
        question(&menus.screens.flow, &tables.pages, &tables.captions),
        leaving(menus),
    );
    let r = f(&mut q, &mut l);
    put_question(&menus.screens.flow, &tables.pages, &tables.captions, &q);
    put_leaving(menus, &l);
    r
}
