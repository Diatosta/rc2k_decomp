//! `marks::run_pong` (0x4B7EDA): the pong game over the race (a reading), with cheat bit
//! 0x8000 (`g_cheats`) and `dword_641678` only: `marks::pong_setup`, `marks::move_pong_bat`,
//! `marks::move_pong_ball` `g_pongSteps` times (read once), `marks::draw_pong_box`
//! for the ball and for the bat (st(0) height, st(1) width, st(2) y, st(3) x), then
//! `marks::pong_score`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BF60F` (0x4BF7CB). The moves are their Rust versions; the set-up, the
//! boxes and the score through `part` (1, 2 with the four values, 3), in the game their
//! Rust versions. Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! set-up, boxes and score go to stand-ins, against the Rust version, balls and bats by
//! lot, the cheat on and off; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::game_read;
use crate::marks::move_pong_ball::move_pong_ball;
use crate::marks::move_pong_bat::move_pong_bat;
use crate::marks::pong;
use crate::scene::matrix3::extended;

const CHEAT: u32 = 0x8000;

fn box_values(place: [usize; 2], size: [usize; 2]) -> [[u8; 10]; 4] {
    unsafe { [size[1], size[0], place[1], place[0]].map(|r| extended(game_read(r))) }
}

/// The Rust version (see the header).
pub unsafe fn run_pong() -> Result<(), Refused> {
    use crate::marks::draw_pong_box::draw_pong_box;
    unsafe {
        if game_read::<u32>(pong::CHEATS_RVA) & CHEAT == 0 || game_read::<u32>(pong::SHOWN_RVA) == 0
        {
            return Ok(());
        }
        crate::marks::pong_setup::pong_setup();
        move_pong_bat();
        let steps = game_read::<i32>(pong::STEPS_RVA);
        for _ in 0..steps.max(0) {
            move_pong_ball();
        }
        draw_pong_box(box_values(pong::BALL, pong::BALL_SIZE));
        draw_pong_box(box_values(pong::BAT, pong::BAT_SIZE));
        crate::marks::pong_score::pong_score()?;
    }
    Ok(())
}
