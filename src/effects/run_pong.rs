//! `effects::run_pong` (0x4B7EDA): the pong game over the race (a reading), with cheat bit
//! 0x8000 (`g_cheats`) and `dword_641678` only: `effects::pong_setup`, `effects::move_pong_bat`,
//! `effects::move_pong_ball` `g_pongSteps` times (read once), `effects::draw_pong_box`
//! for the ball and for the bat (st(0) height, st(1) width, st(2) y, st(3) x), then
//! `effects::pong_score`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BF60F` (0x4BF7CB). The moves are their Rust versions.

use crate::effects::move_pong_ball::move_pong_ball;
use crate::effects::move_pong_bat::move_pong_bat;
use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::extended;
use crate::session::Float;

const CHEAT: u32 = 0x8000;

fn box_values(place: &[Float; 2], size: &[Float; 2]) -> [[u8; 10]; 4] {
    [&size[1], &size[0], &place[1], &place[0]].map(|f| extended(f.bits()))
}

/// The Rust version (see the header).
pub fn run_pong(frame: &Frame, pong: &Pong, fl: &WindscreenFlakes) -> Result<(), Refused> {
    use crate::effects::draw_pong_box::draw_pong_box;
    let p = &pong.state;
    {
        // `dword_641678`: pong shown.
        if CHEAT_VARS.cheats.get() & CHEAT == 0 || frame.frame_work.lead_kept[7].get() == 0 {
            return Ok(());
        }
        crate::effects::pong_setup::pong_setup(pong, fl);
        move_pong_bat(pong);
        let steps = p.steps.get() as i32;
        for _ in 0..steps.max(0) {
            move_pong_ball(pong);
        }
        draw_pong_box(pong, fl, box_values(&p.ball, &p.ball_size));
        draw_pong_box(pong, fl, box_values(&p.bat, &p.bat_size));
        crate::effects::pong_score::pong_score(pong, fl)?;
    }
    Ok(())
}
