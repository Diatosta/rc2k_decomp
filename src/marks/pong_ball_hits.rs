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
//!
//! Converted 2026-10-21: the hook's Rust version is `pong_ball_hits_plain`, through
//! `rc2k_core::marks::pong::Court::ball_hits` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::pong;
use crate::math::x87;

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
