//! `effects::run_pong` (0x4B7EDA): the pong game over the race (an Easter egg: a ball, a bat and a score; seen on the windscreen), with cheat bit
//! 0x8000 (`g_cheats`) and `FrameWork.lead_kept[7]` only: `effects::pong_setup`, `effects::move_pong_bat`,
//! `effects::move_pong_ball` `g_pongSteps` times (read once), `effects::draw_pong_box`
//! for the ball and for the bat (st(0) height, st(1) width, st(2) y, st(3) x), then
//! `effects::pong_score`.
//!
//! Caller: `effects::windscreen_flakes_frame` (0x4BF7CB). The moves are their Rust versions.

use crate::effects::move_pong_ball::move_pong_ball;
use crate::effects::move_pong_bat::move_pong_bat;
use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::extended;
use crate::session::PlainFloat;

const CHEAT: u32 = 0x8000;

fn box_values(place: &[PlainFloat; 2], size: &[PlainFloat; 2]) -> [[u8; 10]; 4] {
    [&size[1], &size[0], &place[1], &place[0]].map(|f| extended(f.get()))
}

pub fn run_pong(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    frame: &Frame,
    pong: &Pong,
    fl: &WindscreenFlakes,
    scene: &crate::scene::owner::Scene,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::draw_pong_box::draw_pong_box;
    let p = &pong.state;
    {
        // `FrameWork.lead_kept[7]`: pong shown.
        if cheats.cheats.get() & CHEAT == 0 || frame.frame_work.get().lead_kept[7].get() == 0 {
            return Ok(());
        }
        crate::effects::pong_setup::pong_setup(buffer, pong, fl, scene);
        move_pong_bat(pong);
        let steps = p.steps.get() as i32;
        for _ in 0..steps.max(0) {
            move_pong_ball(pong, effects);
        }
        draw_pong_box(
            buffer,
            pong,
            fl,
            box_values(&p.ball, &p.ball_size),
            scene,
            effects,
        );
        draw_pong_box(
            buffer,
            pong,
            fl,
            box_values(&p.bat, &p.bat_size),
            scene,
            effects,
        );
        crate::effects::pong_score::pong_score(buffer, pong, fl, scene, effects)?;
    }
    Ok(())
}
