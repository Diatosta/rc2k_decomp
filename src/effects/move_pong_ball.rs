//! `effects::move_pong_ball`: one step of the pong ball: its place plus its way,
//! `effects::pong_ball_hits`; at or left of the left wall it is put at the wall's reset place and
//! its across way turned (the sign bit flipped); at or right of the right wall less its width less
//! a margin the game starts again (`effects::reset_pong`, the pong's reset); at or above the top
//! wall put at its reset place, at or below the bottom wall less its height put at the bottom reset
//! place less its height, its along way turned each time. ("At or" includes an unordered compare.)

use crate::effects::pong::{self};
use crate::effects::pong_ball_hits::pong_ball_hits;
use crate::effects::pong_store::Pong;
use crate::math::x87;
use crate::memory::session::PlainFloat;

/// The court's limits (`PongRules::walls`).
fn limit(pong: &Pong, n: usize) -> f32 {
    pong.rules.walls[n].get()
}

pub fn move_pong_ball(pong: &Pong, effects: &crate::effects::owner::Effects) {
    let p = &pong.state;
    let limit = |n| limit(pong, n);
    for n in 0..2 {
        p.ball[n].set(x87::add(&p.ball[n], &p.ball_way[n]));
    }
    pong_ball_hits(pong, effects);
    let flip = |r: &PlainFloat| r.set(-r.get());
    if x87::compare_status(&p.ball[0], limit(0)) != 0 {
        p.ball[0].set(x87::loaded(limit(1)));
        flip(&p.ball_way[0]);
    }
    let hit = pong::less_compare(
        p.ball[0].get(),
        limit(2),
        p.ball_size[0].get(),
        Some(limit(3)),
    );
    if hit != 0 {
        crate::effects::splines::reset_pong(pong, effects);
    }
    if x87::compare_status(&p.ball[1], limit(4)) != 0 {
        p.ball[1].set(x87::loaded(limit(5)));
        flip(&p.ball_way[1]);
    }
    let hit = pong::less_compare(p.ball[1].get(), limit(6), p.ball_size[1].get(), None);
    if hit != 0 {
        p.ball[1].set(x87::sub(limit(7), &p.ball_size[1]));
        flip(&p.ball_way[1]);
    }
}
