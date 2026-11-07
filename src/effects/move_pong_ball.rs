//! `effects::move_pong_ball` (0x4B7F4F): one step of the pong ball: its place plus its way,
//! `effects::pong_ball_hits`; at or left of `flt_7691A6` it is put at `flt_7691AA` and its
//! across way turned (the sign bit flipped); at or right of `flt_7691AE` less its width
//! less `flt_7691B2` the game starts again (`effects::reset_pong`, the pong's reset); at or
//! above `flt_7691B6` put at `flt_7691BA`, at or below `flt_7691BE` less its height put at
//! `flt_7691C2` less its height, its along way turned each time. ("At or" includes an
//! unordered compare, as the original's tests of C0, C2, C3 together.)
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
//!
//! Caller: `effects::run_pong` (`g_pongSteps` times a frame). The calls are their Rust
//! versions.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::pong::{self, Pong};
use crate::effects::pong_ball_hits::pong_ball_hits;
use crate::math::x87;
use crate::session::Float;

/// The court's limits `flt_7691A6` .. (`PongRules::walls`), as bits.
fn limit(pong: &Pong, n: usize) -> u32 {
    pong.rules.walls[n].bits()
}

/// The Rust version (see the header).
pub fn move_pong_ball(pong: &Pong, effects: &crate::effects::owner::Effects) {
    let p = &pong.state;
    let limit = |n| limit(pong, n);
    for n in 0..2 {
        p.ball[n].set_bits(x87::add(p.ball[n].bits(), p.ball_way[n].bits()));
    }
    pong_ball_hits(pong, effects);
    let flip = |r: &Float| r.set_bits(r.bits() ^ 0x8000_0000);
    if x87::compare_status(p.ball[0].bits(), limit(0)) != 0 {
        p.ball[0].set_bits(x87::loaded(limit(1)));
        flip(&p.ball_way[0]);
    }
    let hit = pong::less_compare(
        p.ball[0].bits(),
        limit(2),
        p.ball_size[0].bits(),
        Some(limit(3)),
    );
    if hit != 0 {
        crate::effects::splines::reset_pong(pong, effects);
    }
    if x87::compare_status(p.ball[1].bits(), limit(4)) != 0 {
        p.ball[1].set_bits(x87::loaded(limit(5)));
        flip(&p.ball_way[1]);
    }
    let hit = pong::less_compare(p.ball[1].bits(), limit(6), p.ball_size[1].bits(), None);
    if hit != 0 {
        p.ball[1].set_bits(x87::sub(limit(7), p.ball_size[1].bits()));
        flip(&p.ball_way[1]);
    }
}
