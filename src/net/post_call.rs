//! `net::post_call` (0x4184F8): with `g_netPosting` 1, the code at eax handed to the
//! receiving thread (`g_netPostedCall`), which calls it and sets `g_netPostedResult` and
//! `g_netPostedAnswered`; meanwhile a tenth of a second's sleep (`system::sleep`) and the
//! window's messages (`system::pump_messages`) until it has answered or the game is
//! closing; then eax = `g_netPostedResult` (also without posting).
//!
//! In:  eax = the code to call
//! Out: eax = the posted call's result; the rest as the sleep and the messages leave them;
//!      the flags of the last test (no caller reads them)
//! DirectPlay's calls that may show a dialog (a modem's, a serial link's) are made on the
//! receiving thread this way (`net::build_address` sets `g_netPosting`). Callers:
//! `net::find_sessions`, `net::host_session`, `net::join_session`, each posting its own
//! piece of code (0x418A43, 0x418C5A, 0x418D77: `net::lobby_calls` posts Rust versions of
//! them). It waits on another thread: ungated. Shadow mode runs the original alone. Its
//! calls through `part`. Self-test: a copy whose five variables are the test's (the
//! receiving thread reads them) and whose calls go to stand-ins (one answering after some
//! turns), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    POSTED, POSTED_ANSWERED, POSTED_RESULT, POSTING, PUMP, Part, QUITTING, SLEEP, a, answer,
    dispatch_lobby, run_both,
};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::post_call", rva(0x4184F8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub posting: u32,
    pub posted: u32,
    pub result: u32,
    pub answered: u32,
    pub quitting: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            posting: a(POSTING),
            posted: a(POSTED),
            result: a(POSTED_RESULT),
            answered: a(POSTED_ANSWERED),
            quitting: a(QUITTING),
        }
    }
}

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

/// The Rust version on the variables in `v`.
pub fn post_call_with(entry: &Regs, v: &Vars, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if get(v.posting) == 1 {
        unsafe { (v.posted as *mut u32).write_volatile(r.eax) };
        loop {
            r.eax = 0;
            let answered =
                unsafe { AtomicU32::from_ptr(v.answered as *mut u32) }.swap(0, Ordering::SeqCst);
            if answered != 0 {
                break;
            }
            r.eax = 100;
            part(SLEEP, &mut r)?;
            part(PUMP, &mut r)?;
            if get(v.quitting) == 1 {
                break;
            }
        }
    }
    r.eax = get(v.result);
    Ok(r)
}

/// The Rust version.
pub fn post_call(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    post_call_with(entry, &Vars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, post_call);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = vec![0u32; 5];
    let at = vars.as_mut_ptr() as u32;
    let v = Vars {
        posting: at,
        posted: at + 4,
        result: at + 8,
        answered: at + 12,
        quitting: at + 16,
    };
    // cmp g_netPosting, 1
    let copy = match crate::net::lobby::copy_vars(
        0x4184F8,
        0x38,
        &[0x83, 0x3D, 0xE4, 0xAC, 0x52, 0x00, 0x01],
        &[0x1F, 0x24],
        &[SLEEP, PUMP],
        &[
            (POSTING, 4, v.posting),
            (POSTED, 4, v.posted),
            (POSTED_RESULT, 4, v.result),
            (POSTED_ANSWERED, 4, v.answered),
            (QUITTING, 4, v.quitting),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    // (posting, answered at once, the turn the answer comes, the turn the game quits)
    for (posting, at_once, answer_turn, quit_turn) in [
        (0u32, 0u32, 9u32, 9u32),
        (1, 1, 9, 9),
        (1, 0, 2, 9),
        (1, 0, 9, 1),
        (2, 0, 9, 9),
    ] {
        run_both(
            &mut t,
            &format!(
                "posting {posting}, answered {at_once}, answer {answer_turn}, quit {quit_turn}"
            ),
            copy.addr(),
            Regs {
                eax: 0x0041_8A43,
                ..Regs::SENTINEL
            },
            &[(at, 20)],
            0,
            &[],
            &move || {
                let p =
                    move |k: u32, x: u32| unsafe { ((at + 4 * k) as *mut u32).write_volatile(x) };
                p(0, posting);
                p(1, 0);
                p(2, 0x1234);
                p(3, at_once);
                p(4, 0);
                let mut turn = 0;
                answer(Box::new(move |place, r| {
                    if place == PUMP {
                        turn += 1;
                        if turn == answer_turn {
                            p(3, 1);
                        }
                        if turn == quit_turn {
                            p(4, 1);
                        }
                        r.ecx = turn;
                    }
                }));
            },
            &|r, part| post_call_with(r, &v, part),
        );
    }
    let _ = &mut vars;
    t.finish()
}
