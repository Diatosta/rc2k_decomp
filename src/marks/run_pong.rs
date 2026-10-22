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

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::move_pong_ball::move_pong_ball;
use crate::marks::move_pong_bat::move_pong_bat;
use crate::marks::pong;
use crate::scene::matrix3::extended;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_pong", rva(0x4B7EDA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_pong(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { run_pong(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(pong::note_plain);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4B7EDA)),
            0x75,
            // test g_cheats, 8000h
            &[0xF7, 0x05, 0x0C, 0x87, 0x75, 0x00, 0x00, 0x80, 0x00, 0x00],
            &[0x15, 0x1A, 0x27, 0x4D, 0x6A, 0x6F],
            &|c| match c + 0x40_0000 {
                0x4B88CB => Some(stand_in(1)),
                0x4B8599 => Some(pong::box_stand_in as *const () as usize),
                0x4B8382 => Some(stand_in(3)),
                _ => None,
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = pong::regions();
    let mut vars = regions.clone();
    vars.extend([pong::CHEATS_RVA, pong::SHOWN_RVA].map(at));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..6u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            pong::test_values(case);
            let cheats: u32 = game_read(pong::CHEATS_RVA);
            game_write(
                pong::CHEATS_RVA,
                if case == 4 {
                    cheats & !CHEAT
                } else {
                    cheats | CHEAT
                },
            );
            game_write(pong::SHOWN_RVA, u32::from(case != 5));
        };
        prepare();
        pong::take();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), pong::take(), capture(&regions));
        prepare();
        let mine = unsafe {
            run_pong(&Regs::SENTINEL, &mut |p, r, v| {
                pong::note(p, r, v);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            pong::take(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {} calls; rust {:x?} {} calls; same calls {}; same state {}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.1 == mine.1,
                theirs.2 == mine.2
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
