//! `marks::pong_ball_hits` (0x4B8040): when the pong ball overlaps the bat (`marks::pong`'s
//! four compares): a hit (`dword_7687D0` plus 1), four more steps a frame
//! (`g_pongSteps`), the ball a step back, and its new way: from the bat's centre to
//! the ball's (`flt_7686DC` .. `flt_7686F8`; an across of exactly 0 bits less
//! `flt_7691D6`), made one long times `flt_76516C`, times `flt_7691DA` / `flt_7691DE`.
//!
//! In:  nothing
//! Out: eax the last compare's status (& 0x700), or after a hit eax / ebx the new way's
//!      bits
//!
//! Caller: `marks::move_pong_ball`. The x87 runs are the original's instructions over the
//! game's variables. Self-test: the original and the Rust version on balls and bats by
//! lot (hits among them); put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::pong;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::pong_ball_hits", rva(0x4B8040), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The original's 0x4B8105 .. 0x4B8189: the centres and their difference.
unsafe fn centres(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x3686A0]",
            "fmul dword ptr [{b} + 0x3691C6]",
            "fadd dword ptr [{b} + 0x368688]",
            "fstp dword ptr [{b} + 0x3686DC]",
            "fld dword ptr [{b} + 0x3686A4]",
            "fmul dword ptr [{b} + 0x3691CA]",
            "fadd dword ptr [{b} + 0x36868C]",
            "fstp dword ptr [{b} + 0x3686E0]",
            "fld dword ptr [{b} + 0x3686D0]",
            "fmul dword ptr [{b} + 0x3691CE]",
            "fadd dword ptr [{b} + 0x3686B8]",
            "fstp dword ptr [{b} + 0x3686E8]",
            "fld dword ptr [{b} + 0x3686D4]",
            "fmul dword ptr [{b} + 0x3691D2]",
            "fadd dword ptr [{b} + 0x3686BC]",
            "fstp dword ptr [{b} + 0x3686EC]",
            "fld dword ptr [{b} + 0x3686DC]",
            "fsub dword ptr [{b} + 0x3686E8]",
            "fstp dword ptr [{b} + 0x3686F4]",
            "fld dword ptr [{b} + 0x3686E0]",
            "fsub dword ptr [{b} + 0x3686EC]",
            "fstp dword ptr [{b} + 0x3686F8]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B81A4 .. 0x4B81FE: the difference made one long, scaled.
unsafe fn new_way(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x3686F4]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{b} + 0x3686F8]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{b} + 0x36516C]",
            "fld dword ptr [{b} + 0x3686F4]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{b} + 0x3686F4]",
            "fld dword ptr [{b} + 0x3686F8]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{b} + 0x3686F8]",
            "fld dword ptr [{b} + 0x3686F4]",
            "fmul dword ptr [{b} + 0x3691DA]",
            "fstp dword ptr [{b} + 0x3686F4]",
            "fld dword ptr [{b} + 0x3686F8]",
            "fmul dword ptr [{b} + 0x3691DE]",
            "fstp dword ptr [{b} + 0x3686F8]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

const ACROSS_RVA: usize = rva(0x7686F4); // flt_7686F4
const ALONG_RVA: usize = rva(0x7686F8); // flt_7686F8
const NUDGE_RVA: usize = rva(0x7691D6); // flt_7691D6

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pong_ball_hits(entry: &Regs) -> Regs {
    let mut g = *entry;
    let (over, status) = pong::overlap();
    g.eax = status;
    if !over {
        return g;
    }
    unsafe {
        game_write(
            pong::HITS_RVA,
            game_read::<u32>(pong::HITS_RVA).wrapping_add(1),
        );
        game_write(
            pong::STEPS_RVA,
            game_read::<u32>(pong::STEPS_RVA).wrapping_add(4),
        );
        for n in 0..2 {
            let v = x87::sub(game_read(pong::BALL[n]), game_read(pong::BALL_WAY[n]));
            game_write(pong::BALL[n], v);
        }
        let b = game_addr(0) as u32;
        centres(b);
        if game_read::<u32>(ACROSS_RVA) == 0 {
            game_write(
                ACROSS_RVA,
                x87::sub(game_read(ACROSS_RVA), game_read(NUDGE_RVA)),
            );
        }
        new_way(b);
        g.eax = game_read(ACROSS_RVA);
        g.ebx = game_read(ALONG_RVA);
        game_write(pong::BALL_WAY[0], g.eax);
        game_write(pong::BALL_WAY[1], g.ebx);
    }
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pong_ball_hits(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &pong::regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = pong::regions();
    let vars = regions.clone();
    let saved = capture(&vars);
    for case in 0..8u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                pong::test_values(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
