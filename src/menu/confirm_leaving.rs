//! `menu::confirm_leaving` (0x46FA90): asks whether to leave.
//!
//! In:  nothing
//! Out: the registers as `menu::leave_question_loop` leaves them; the flags of
//!      `cmp g_yesNoAnswer, 1`
//!
//! `g_yesNoAnswer` (0x75524C) = 0; `menu::leave_question_loop` (the "leave?" screen);
//! if that left `dword_758806` at 1: `dword_75555D`, `dword_755561`, `dword_758806` and
//! the answer 0; then with the answer at 1, `dword_75555D` and `dword_755561` = 1.
//! 37 callers (the screen loops that ask before leaving).
//! Shadow mode runs the original alone (the question is a screen).
//! Self-test: a redirected copy whose question is a stand-in that leaves the answer
//! and `dword_758806` as the case says.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::confirm_leaving", rva(0x46FA90), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const GO_ON_RVA: usize = rva(0x75555D); // dword_75555D
const LEAVE_RVA: usize = rva(0x755561); // dword_755561
const SET_UP_AGAIN_RVA: usize = rva(0x758806); // dword_758806
const QUESTION_RVA: usize = rva(0x47FA94); // menu::leave_question_loop

// ---- Rust version ----

/// `ask` stands for the question's screen.
pub unsafe fn confirm_leaving(r: &mut Regs, ask: &mut dyn FnMut(&mut Regs)) {
    unsafe {
        game_write(ANSWER_RVA, 0u32);
        ask(r);
        if game_read::<u32>(SET_UP_AGAIN_RVA) == 1 {
            game_write(GO_ON_RVA, 0u32);
            game_write(LEAVE_RVA, 0u32);
            game_write(SET_UP_AGAIN_RVA, 0u32);
            game_write(ANSWER_RVA, 0u32);
        }
        let answer: u32 = game_read(ANSWER_RVA);
        if answer == 1 {
            game_write(GO_ON_RVA, 1u32);
            game_write(LEAVE_RVA, 1u32);
        }
        r.set_carry(answer < 1);
        r.set_zero(answer == 1);
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { confirm_leaving(r, &mut |r| call_game(r, game_addr(QUESTION_RVA))) };
}

// ---- self-test ----

const LEN: usize = 0x5E;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0x4C, 0x52, 0x75, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 1] = [0x0A];

/// What the stand-in leaves: the answer and `dword_758806`.
static LEAVES: AtomicU32 = AtomicU32::new(0);
static GIVEN: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, r: &mut Regs) {
    unsafe {
        // What the answer was when the question began.
        GIVEN.store(game_read(ANSWER_RVA), Ordering::SeqCst);
        let leaves = LEAVES.load(Ordering::SeqCst);
        game_write(ANSWER_RVA, leaves & 0xFF);
        game_write(SET_UP_AGAIN_RVA, leaves >> 8);
    }
    r.eax = 0x1234;
    r.esi ^= 0x55;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x46FA90)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == QUESTION_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 4] = [
        at(ANSWER_RVA),
        at(GO_ON_RVA),
        at(LEAVE_RVA),
        at(SET_UP_AGAIN_RVA),
    ];
    let saved = capture(&regions);
    set_handler(handler);
    for answer in [0u32, 1, 2, 0xFF] {
        for again in [0u32, 1, 2] {
            for before_state in [0u32, 1] {
                LEAVES.store(answer | again << 8, Ordering::SeqCst);
                let run = |rust: bool| unsafe {
                    for region in regions.iter().skip(1) {
                        (region.0 as *mut u32).write_unaligned(before_state * 7);
                    }
                    game_write(ANSWER_RVA, 9u32);
                    GIVEN.store(0xFFFF_FFFF, Ordering::SeqCst);
                    let before = Regs::SENTINEL;
                    let mut r = before;
                    if rust {
                        confirm_leaving(&mut r, &mut |r| act(0, r));
                    } else {
                        call_game(&mut r, copy.addr());
                    }
                    (
                        [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                        r.carry(),
                        r.eflags & 0x40 != 0,
                        capture(&regions),
                        GIVEN.load(Ordering::SeqCst),
                    )
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine, || {
                    format!(
                        "answer {answer:#x}, dword_758806 {again}, before {before_state}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    restore(&regions, &saved);
    t.finish()
}
