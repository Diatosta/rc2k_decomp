//! `marks::run_pong` (0x4B7EDA): the pong game over the race (a reading), with cheat bit
//! 0x8000 (`g_cheats`) and `dword_641678` only: `marks::pong_setup`, `marks::move_pong_bat`,
//! `marks::move_pong_ball` `g_pongSteps` times (read once), `marks::draw_pong_box`
//! for the ball and for the bat (st(0) height, st(1) width, st(2) y, st(3) x), then
//! `marks::pong_score`.
//!
//! In:  nothing
//! Out: as the path leaves them
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
use crate::framework::{Regs, game_read};
use crate::marks::move_pong_ball::move_pong_ball;
use crate::marks::move_pong_bat::move_pong_bat;
use crate::marks::pong;
use crate::scene::matrix3::extended;

const CHEAT: u32 = 0x8000;

/// A call by place (see the header), with its x87 values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

fn box_values(place: [usize; 2], size: [usize; 2]) -> [[u8; 10]; 4] {
    unsafe { [size[1], size[0], place[1], place[0]].map(|r| extended(game_read(r))) }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_pong(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(pong::CHEATS_RVA) & CHEAT == 0 || game_read::<u32>(pong::SHOWN_RVA) == 0
        {
            return Ok(g);
        }
        part(1, &mut g, &[])?;
        g = move_pong_bat(&g);
        let steps = game_read::<i32>(pong::STEPS_RVA);
        for _ in 0..steps.max(0) {
            g = move_pong_ball(&g);
        }
        part(2, &mut g, &box_values(pong::BALL, pong::BALL_SIZE))?;
        part(2, &mut g, &box_values(pong::BAT, pong::BAT_SIZE))?;
        part(3, &mut g, &[])?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::{draw_pong_box as pb, pong_score as ps, pong_setup, squares_setup};
    unsafe {
        *r = match place {
            1 => pong_setup::pong_setup(r, &mut squares_setup::game_part)?,
            2 => pb::draw_pong_box(r, [v[0], v[1], v[2], v[3]], &mut pb::game_part)?,
            _ => ps::pong_score(r, &mut ps::game_part)?,
        };
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { run_pong(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
