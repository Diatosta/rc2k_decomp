//! The game's leaving switches and yes/no question as `rc2k_core::menu::leaving`'s
//! `Leaving` and `Question`: read, changed, written back whole.

use crate::framework::{game_read, game_write, rva};
use rc2k_core::menu::leaving::{Leaving, Question};

pub const LEAVE_EVENT_RVA: usize = rva(0x75555D); // dword_75555D
pub const LEAVE_SCREEN_RVA: usize = rva(0x755561); // dword_755561
pub const LEAVE_TOO_RVA: usize = rva(0x755565); // dword_755565
pub const ASK_RVA: usize = rva(0x755569); // dword_755569
pub const FIELD_A0C_RVA: usize = rva(0x755A0C); // dword_755A0C
pub const ASKING_RVA: usize = rva(0x7554CC); // dword_7554CC: 1 on a yes/no screen
pub const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer: 1 = yes, 0 = no
pub const ANSWERED_RVA: usize = rva(0x758806); // dword_758806

const LEAVING: [usize; 5] = [
    LEAVE_EVENT_RVA,
    LEAVE_SCREEN_RVA,
    LEAVE_TOO_RVA,
    ASK_RVA,
    FIELD_A0C_RVA,
];
const QUESTION: [usize; 3] = [ASKING_RVA, ANSWER_RVA, ANSWERED_RVA];

fn get<const N: usize>(at: [usize; N]) -> [u32; N] {
    at.map(|a| unsafe { game_read(a) })
}

fn put<const N: usize>(at: [usize; N], v: [u32; N]) {
    for (a, v) in at.into_iter().zip(v) {
        unsafe { game_write(a, v) };
    }
}

pub fn leaving() -> Leaving {
    let [leave_event, leave_screen, leave_too, ask, field_a0c] = get(LEAVING);
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
        LEAVING,
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
    let [asking, answer, answered] = get(QUESTION);
    Question {
        asking,
        answer,
        answered,
    }
}

pub fn put_question(q: &Question) {
    put(QUESTION, [q.asking, q.answer, q.answered]);
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
