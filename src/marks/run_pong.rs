//! `marks::run_pong` (0x4B7EDA): the pong game over the race (a reading), with cheat bit
//! 0x8000 (`g_cheats`) and `dword_641678` only: `marks::pong_setup`, `marks::move_pong_bat`,
//! `marks::move_pong_ball` `g_pongSteps` times (read once), `marks::draw_pong_box`
//! for the ball and for the bat (st(0) height, st(1) width, st(2) y, st(3) x), then
//! `marks::pong_score`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BF60F` (0x4BF7CB). The moves are their Rust versions.

use crate::errors::Refused;
use crate::marks::move_pong_ball::move_pong_ball;
use crate::marks::move_pong_bat::move_pong_bat;
use crate::marks::pong;
use crate::marks::store::game_read;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::scene::matrix3::extended;
use crate::session::Float;

const CHEAT: u32 = 0x8000;

fn box_values(place: &[Float; 2], size: &[Float; 2]) -> [[u8; 10]; 4] {
    [&size[1], &size[0], &place[1], &place[0]].map(|f| extended(f.bits()))
}

/// The Rust version (see the header).
pub fn run_pong() -> Result<(), Refused> {
    use crate::marks::draw_pong_box::draw_pong_box;
    let p = &pong::PONG;
    {
        if CHEAT_VARS.cheats.get() & CHEAT == 0 || game_read::<u32>(pong::SHOWN_RVA) == 0 {
            return Ok(());
        }
        crate::marks::pong_setup::pong_setup();
        move_pong_bat();
        let steps = p.steps.get() as i32;
        for _ in 0..steps.max(0) {
            move_pong_ball();
        }
        draw_pong_box(box_values(&p.ball, &p.ball_size));
        draw_pong_box(box_values(&p.bat, &p.bat_size));
        crate::marks::pong_score::pong_score()?;
    }
    Ok(())
}
