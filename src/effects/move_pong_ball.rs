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

/// The court's limits (`PongRules::walls`).
fn limit(pong: &Pong, n: usize) -> f32 {
    pong.rules.walls[n]
}

pub fn move_pong_ball(pong: &Pong, effects: &crate::effects::owner::Effects) {
    let limit = |n| limit(pong, n);
    let flip = |v: f32| -v;
    {
        let mut p = pong.state.borrow_mut();
        for n in 0..2 {
            p.ball[n] = x87::add(p.ball[n], p.ball_way[n]);
        }
    }
    pong_ball_hits(pong, effects);
    let (ball0, size0) = {
        let p = pong.state.borrow();
        (p.ball[0], p.ball_size[0])
    };
    if x87::compare_status(ball0, limit(0)) != 0 {
        let mut p = pong.state.borrow_mut();
        p.ball[0] = x87::loaded(limit(1));
        p.ball_way[0] = flip(p.ball_way[0]);
    }
    let ball0 = pong.state.borrow().ball[0];
    let hit = pong::less_compare(ball0, limit(2), size0, Some(limit(3)));
    if hit != 0 {
        crate::effects::splines::reset_pong(pong, effects);
    }
    let (ball1, size1) = {
        let p = pong.state.borrow();
        (p.ball[1], p.ball_size[1])
    };
    if x87::compare_status(ball1, limit(4)) != 0 {
        let mut p = pong.state.borrow_mut();
        p.ball[1] = x87::loaded(limit(5));
        p.ball_way[1] = flip(p.ball_way[1]);
    }
    let ball1 = pong.state.borrow().ball[1];
    let hit = pong::less_compare(ball1, limit(6), size1, None);
    if hit != 0 {
        let mut p = pong.state.borrow_mut();
        p.ball[1] = x87::sub(limit(7), size1);
        p.ball_way[1] = flip(p.ball_way[1]);
    }
}
