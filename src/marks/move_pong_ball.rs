//! `marks::move_pong_ball` (0x4B7F4F): one step of the pong ball: its place plus its way,
//! `marks::pong_ball_hits`; at or left of `flt_7691A6` it is put at `flt_7691AA` and its
//! across way turned (the sign bit flipped); at or right of `flt_7691AE` less its width
//! less `flt_7691B2` the game starts again (`marks::reset_pong`, the pong's reset); at or
//! above `flt_7691B6` put at `flt_7691BA`, at or below `flt_7691BE` less its height put at
//! `flt_7691C2` less its height, its along way turned each time. ("At or" includes an
//! unordered compare, as the original's tests of C0, C2, C3 together.)
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
//!
//! Caller: `marks::run_pong` (`g_pongSteps` times a frame). The calls are their Rust
//! versions.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::pong;
use crate::marks::pong_ball_hits::pong_ball_hits;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const LIMITS_RVA: Slot = Slot::at(0x7691A6); // flt_7691A6 .. flt_7691C2

fn limit(n: usize) -> u32 {
    unsafe { game_read(LIMITS_RVA + 4 * n) }
}

/// The Rust version (see the header).
pub unsafe fn move_pong_ball() {
    unsafe {
        for n in 0..2 {
            let v = x87::add(game_read(pong::BALL[n]), game_read(pong::BALL_WAY[n]));
            game_write(pong::BALL[n], v);
        }
        pong_ball_hits();
        let flip = |r: Slot| game_write(r, game_read::<u32>(r) ^ 0x8000_0000);
        if x87::compare_status(game_read(pong::BALL[0]), limit(0)) != 0 {
            game_write(pong::BALL[0], x87::loaded(limit(1)));
            flip(pong::BALL_WAY[0]);
        }
        let hit = pong::less_compare(
            game_read(pong::BALL[0]),
            limit(2),
            game_read(pong::BALL_SIZE[0]),
            Some(limit(3)),
        );
        if hit != 0 {
            crate::marks::splines::reset_pong();
        }
        if x87::compare_status(game_read(pong::BALL[1]), limit(4)) != 0 {
            game_write(pong::BALL[1], x87::loaded(limit(5)));
            flip(pong::BALL_WAY[1]);
        }
        let hit = pong::less_compare(
            game_read(pong::BALL[1]),
            limit(6),
            game_read(pong::BALL_SIZE[1]),
            None,
        );
        if hit != 0 {
            game_write(
                pong::BALL[1],
                x87::sub(limit(7), game_read(pong::BALL_SIZE[1])),
            );
            flip(pong::BALL_WAY[1]);
        }
    }
}
