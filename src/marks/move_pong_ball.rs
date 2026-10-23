//! `marks::move_pong_ball` (0x4B7F4F): one step of the pong ball: its place plus its way,
//! `marks::pong_ball_hits`; at or left of `flt_7691A6` it is put at `flt_7691AA` and its
//! across way turned (the sign bit flipped); at or right of `flt_7691AE` less its width
//! less `flt_7691B2` the game starts again (`marks::reset_pong`, the pong's reset); at or
//! above `flt_7691B6` put at `flt_7691BA`, at or below `flt_7691BE` less its height put at
//! `flt_7691C2` less its height, its along way turned each time. ("At or" includes an
//! unordered compare, as the original's tests of C0, C2, C3 together.)
//!
//! In:  nothing
//! Out: eax the last compare's status (& 0x700); the rest as the hit test leaves them
//!
//! Caller: `marks::run_pong` (`g_pongSteps` times a frame). The calls are their
//! Rust versions. Self-test: the original and the Rust version on balls and bats by lot,
//! walls hit among them; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `move_pong_ball_plain`, through
//! `rc2k_core::marks::pong::Court::move_ball` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::pong;
use crate::marks::pong_ball_hits::pong_ball_hits;
use crate::math::x87;
use rc2k_core::marks::pong::Hit;

const LIMITS_RVA: usize = rva(0x7691A6); // flt_7691A6 .. flt_7691C2

fn limit(n: usize) -> u32 {
    unsafe { game_read(LIMITS_RVA + 4 * n) }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn move_pong_ball(entry: &Regs) -> Regs {
    unsafe {
        for n in 0..2 {
            let v = x87::add(game_read(pong::BALL[n]), game_read(pong::BALL_WAY[n]));
            game_write(pong::BALL[n], v);
        }
        let mut g = pong_ball_hits(entry);
        let flip = |r: usize| game_write(r, game_read::<u32>(r) ^ 0x8000_0000);
        g.eax = x87::compare_status(game_read(pong::BALL[0]), limit(0)) as u32;
        if g.eax != 0 {
            game_write(pong::BALL[0], x87::loaded(limit(1)));
            flip(pong::BALL_WAY[0]);
        }
        g.eax = pong::less_compare(
            game_read(pong::BALL[0]),
            limit(2),
            game_read(pong::BALL_SIZE[0]),
            Some(limit(3)),
        );
        if g.eax != 0 {
            crate::marks::splines::reset_pong();
        }
        g.eax = x87::compare_status(game_read(pong::BALL[1]), limit(4)) as u32;
        if g.eax != 0 {
            game_write(pong::BALL[1], x87::loaded(limit(5)));
            flip(pong::BALL_WAY[1]);
        }
        g.eax = pong::less_compare(
            game_read(pong::BALL[1]),
            limit(6),
            game_read(pong::BALL_SIZE[1]),
            None,
        );
        if g.eax != 0 {
            game_write(
                pong::BALL[1],
                x87::sub(limit(7), game_read(pong::BALL_SIZE[1])),
            );
            flip(pong::BALL_WAY[1]);
        }
        g
    }
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::pong::Court::move_ball`
pub unsafe fn move_pong_ball_plain(entry: &Regs) -> Regs {
    let mut g = *entry;
    let rules = pong::rules();
    let (hit, status) = pong::with_court(|c| c.move_ball(&rules));
    if let Hit::Hit([x, y]) = hit {
        (g.eax, g.ebx) = (x, y);
    }
    g.eax = status;
    g
}
