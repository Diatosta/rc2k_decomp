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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::pong;
use crate::marks::pong_ball_hits::pong_ball_hits;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::move_pong_ball", rva(0x4B7F4F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { move_pong_ball(r) }))
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
    for case in 0..10u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                pong::test_values(case);
                if case >= 6 {
                    // walls: the ball put at or past the limits
                    let k = case - 6;
                    let at_limit = |n: usize, d: f32| f32::from_bits(limit(n)) + d;
                    game_write(
                        pong::BALL[0],
                        at_limit(
                            if k % 2 == 0 { 0 } else { 2 },
                            if k < 2 { -1.0 } else { 40.0 },
                        )
                        .to_bits(),
                    );
                    game_write(
                        pong::BALL[1],
                        at_limit(
                            if k % 2 == 0 { 4 } else { 6 },
                            if k < 2 { -2.0 } else { 3.0 },
                        )
                        .to_bits(),
                    );
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
