//! `marks::pong_ball_hits` (0x4B8040): when the pong ball overlaps the bat (`marks::pong`'s
//! four compares): a hit (`dword_7687D0` plus 1), four more steps a frame
//! (`g_pongSteps`), the ball a step back, and its new way: from the bat's centre to
//! the ball's (`flt_7686DC` .. `flt_7686F8`; an across of exactly 0 bits less
//! `flt_7691D6`), made one long times `flt_76516C`, times `flt_7691DA` / `flt_7691DE`.
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
//!
//! Caller: `marks::move_pong_ball`. The x87 runs are the original's instructions over the
//! game's variables.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::pong;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

/// The original's 0x4B8105 .. 0x4B8189: the centres and their difference.
unsafe fn centres() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0x3760]",
            "fmul dword ptr [{mk} + 0x4286]",
            "fadd dword ptr [{mk} + 0x3748]",
            "fstp dword ptr [{mk} + 0x379C]",
            "fld dword ptr [{mk} + 0x3764]",
            "fmul dword ptr [{mk} + 0x428A]",
            "fadd dword ptr [{mk} + 0x374C]",
            "fstp dword ptr [{mk} + 0x37A0]",
            "fld dword ptr [{mk} + 0x3790]",
            "fmul dword ptr [{mk} + 0x428E]",
            "fadd dword ptr [{mk} + 0x3778]",
            "fstp dword ptr [{mk} + 0x37A8]",
            "fld dword ptr [{mk} + 0x3794]",
            "fmul dword ptr [{mk} + 0x4292]",
            "fadd dword ptr [{mk} + 0x377C]",
            "fstp dword ptr [{mk} + 0x37AC]",
            "fld dword ptr [{mk} + 0x379C]",
            "fsub dword ptr [{mk} + 0x37A8]",
            "fstp dword ptr [{mk} + 0x37B4]",
            "fld dword ptr [{mk} + 0x37A0]",
            "fsub dword ptr [{mk} + 0x37AC]",
            "fstp dword ptr [{mk} + 0x37B8]",
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B81A4 .. 0x4B81FE: the difference made one long, scaled.
unsafe fn new_way() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0x37B4]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x37B8]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{mk} + 0x37B4]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x37B4]",
            "fld dword ptr [{mk} + 0x37B8]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x37B8]",
            "fld dword ptr [{mk} + 0x37B4]",
            "fmul dword ptr [{mk} + 0x429A]",
            "fstp dword ptr [{mk} + 0x37B4]",
            "fld dword ptr [{mk} + 0x37B8]",
            "fmul dword ptr [{mk} + 0x429E]",
            "fstp dword ptr [{mk} + 0x37B8]",
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

const ACROSS_RVA: Slot = Slot::at(0x7686F4); // flt_7686F4
const ALONG_RVA: Slot = Slot::at(0x7686F8); // flt_7686F8
const NUDGE_RVA: Slot = Slot::at(0x7691D6); // flt_7691D6

/// The Rust version (see the header).
pub unsafe fn pong_ball_hits() {
    let (over, _) = pong::overlap();
    if !over {
        return;
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
        centres();
        if game_read::<u32>(ACROSS_RVA) == 0 {
            game_write(
                ACROSS_RVA,
                x87::sub(game_read(ACROSS_RVA), game_read(NUDGE_RVA)),
            );
        }
        new_way();
        game_write(pong::BALL_WAY[0], game_read::<u32>(ACROSS_RVA));
        game_write(pong::BALL_WAY[1], game_read::<u32>(ALONG_RVA));
    }
}
