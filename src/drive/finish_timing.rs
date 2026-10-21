//! `drive::finish_timing` (0x45948A): the finish routine's time and shares. The clock
//! (`scene::restart_clock`) less the start `dword_64BB60` / `flt_5C5D18` into
//! `flt_64BB64` (t); the eased time `flt_64BB68`: t below `flt_64BB44` (or unordered) t;
//! below `flt_64BB48` t less ((t - a) / (b - a))^2 x (b - a) / 2; else a + (b - a) / 2;
//! the share `flt_64BB6C`: 0, with `dword_75B108` and t from `flt_64BB70` on 1, below
//! `flt_64BB74` (t - c) / (d - c); and with `dword_75B108` and t from `flt_64BB70` on,
//! `dword_5C2F88` 1.
//!
//! In:  nothing
//! Out: eax the clock less the start; the rest kept; no flags read
//!
//! Caller: `drive::finish_run`. It begins with a call: hooked by a private copy of its
//! 0x144 bytes (the call relocated) and a hand-made patch.
//! Shadow mode runs the original alone (the clock moves). Self-test: a copy whose clock
//! call goes to a stand-in giving the test's times, against the Rust version with the
//! same, under three precisions.

use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::math::x87::loaded;
use crate::physics::flow::{a, base, c0, g, p, x87_run};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, FLOAT, Floats, at_test_precisions, compare_memory, note_drift,
};
use crate::service::harness::{capture, restore};
use rc2k_core::drive::finish_timing::FinishFigures;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::finish_timing", rva(0x45948A), thunk, &ORIGINAL)
    .private_copy(0x144)
    .relocate_calls(&[0x0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const T: usize = 0x64BB64;

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn timing_from(now: u32) -> Left {
    let since = now.wrapping_sub(g(0x64BB60));
    p(0x62400C, since);
    x87_run!(
        "fild dword ptr [{b} + 0x22400C]",
        "fdiv dword ptr [{b} + 0x1C5D18]",
        "fstp dword ptr [{b} + 0x24BB64]";
        b = base()
    );
    if c0(g(T), g(0x64BB44)) {
        p(0x64BB68, loaded(g(T)));
    } else if c0(g(T), g(0x64BB48)) {
        x87_run!(
            "fld dword ptr [{b} + 0x24BB64]",
            "fsub dword ptr [{b} + 0x24BB44]",
            "fld dword ptr [{b} + 0x24BB48]",
            "fsub dword ptr [{b} + 0x24BB44]",
            ".byte 0xDE, 0xF9", // fdivp st(1), st
            ".byte 0xD8, 0xC8", // fmul st, st
            "fld dword ptr [{b} + 0x24BB48]",
            "fsub dword ptr [{b} + 0x24BB44]",
            "fmul dword ptr [{b} + 0x1C5E40]",
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            ".byte 0xD9, 0xE0", // fchs
            "fadd dword ptr [{b} + 0x24BB64]",
            "fstp dword ptr [{b} + 0x24BB68]";
            b = base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{b} + 0x24BB48]",
            "fsub dword ptr [{b} + 0x24BB44]",
            "fmul dword ptr [{b} + 0x1C5E40]",
            "fadd dword ptr [{b} + 0x24BB44]",
            "fstp dword ptr [{b} + 0x24BB68]";
            b = base()
        );
    }
    p(0x64BB6C, 0);
    let late = !c0(g(T), g(0x64BB70));
    if g(0x75B108) != 0 && late {
        p(0x64BB6C, 0x3F80_0000);
        if c0(g(T), g(0x64BB74)) {
            x87_run!(
                "fld dword ptr [{b} + 0x24BB64]",
                "fsub dword ptr [{b} + 0x24BB70]",
                "fld dword ptr [{b} + 0x24BB74]",
                "fsub dword ptr [{b} + 0x24BB70]",
                ".byte 0xDE, 0xF9", // fdivp st(1), st
                "fstp dword ptr [{b} + 0x24BB6C]";
                b = base()
            );
        }
    }
    if late && g(0x75B108) != 0 {
        p(0x5C2F88, 1);
    }
    Left::nothing().with(EAX, since)
}

/// The converted core (`rc2k_core::drive::finish_timing`, plain floats) from the clock's
/// reading `now`.
// Converted: `rc2k_core::drive::finish_timing::finish_timing`.
pub fn timing_from_plain(now: u32) -> Left {
    let since = now.wrapping_sub(g(0x64BB60));
    p(0x62400C, since);
    let f = |x: usize| f32::from_bits(g(x));
    let out = rc2k_core::drive::finish_timing::finish_timing(
        since,
        &FinishFigures {
            units: f(0x5C5D18),
            ease: [f(0x64BB44), f(0x64BB48)],
            half: f(0x5C5E40),
            share: [f(0x64BB70), f(0x64BB74)],
            switch: g(0x75B108) != 0,
        },
    );
    p(T, out.time.to_bits());
    p(0x64BB68, out.eased.to_bits());
    p(0x64BB6C, out.share.to_bits());
    if out.flag {
        p(0x5C2F88, 1);
    }
    Left::nothing().with(EAX, since)
}

/// The Rust version (the exact one, for `drive::finish_run`).
pub unsafe fn finish_timing() -> Left {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    timing_from(now)
}

/// The converted version: the hook's.
pub unsafe fn finish_timing_plain() -> Left {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    timing_from_plain(now)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { finish_timing_plain() }.apply(r);
}

static NOW: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, r: &mut Regs) {
    let now = NOW.load(Ordering::SeqCst);
    p(0x613830, now);
    r.eax = now;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x45948A)),
            0x144,
            &[0xE8, 0xF2, 0x6E, 0xFF, 0xFF],
            &[0x00],
            &|_| Some(stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let all = [
        (a(0x613830), 4),
        (a(0x62400C), 4),
        (a(0x64BB44), 0x34),
        (a(0x75B108), 4),
        (a(0x5C2F88), 4),
    ];
    let saved = capture(&all);
    let set_figures = || {
        p(0x64BB60, 1000);
        for (x, v) in [
            (0x64BB44usize, 2.0f32),
            (0x64BB48, 5.0),
            (0x64BB70, 3.0),
            (0x64BB74, 6.0),
        ] {
            p(x, v.to_bits());
        }
        p(0x5C2F88, 0);
    };
    let tolerant = Floats {
        memory: &[((a(T), 12), FLOAT)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for (now, flag) in [
            (1100u32, 1u32),
            (1300, 1),
            (1350, 0),
            (1450, 1),
            (1800, 1),
            (3000, 1),
        ] {
            NOW.store(now, Ordering::SeqCst);
            set_figures();
            p(0x75B108, flag);
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = (ro.eax, capture(&all));
            set_figures();
            p(0x75B108, flag);
            p(0x613830, now);
            let mine = timing_from_plain(now);
            let state = capture(&all);
            let agree = compare_memory(&all, &theirs.1, &state, floats);
            if let Agreement::Drift(n) = agree {
                note_drift("drive::finish_timing", || {
                    format!("self-test now {now}: {n} places")
                });
            }
            t.check(mine == Left::nothing().with(EAX, theirs.0) && agree != Agreement::Differ, || {
                    format!("control word {word:#x}, now {now}, flag {flag}: original eax {:x}; rust {mine:?}", theirs.0)
                });
        }
    });
    restore(&all, &saved);
    t.finish()
}
