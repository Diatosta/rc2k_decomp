//! Frame callbacks that only call, in order, a few of the game's functions, some of which
//! end the frame with a set carry (`menu::picture_frame`, `menu::fade_frame`): the steps,
//! and their test (a copy whose calls go to `menu::frame_test`'s stand-ins, each failing
//! in turn, against the Rust version on the same).

use crate::framework::{Regs, SelfTest, call_game, game_addr, rva};
use crate::menu::frame_test::{self, Noting};
use crate::menu::screen_loop::Env;
use crate::redirect::RedirectedCopy;

pub const LOCK_SURFACE: usize = rva(0x4058AD); // gfx::lock_surface
pub const UNLOCK_SURFACE: usize = rva(0x405949); // gfx::unlock_surface
pub const COPY_PICTURE: usize = rva(0x4060E3); // gfx::copy_picture_to_surface
pub const BEGIN_SCENE: usize = rva(0x44E8A1); // scene::begin_scene
pub const END_SCENE: usize = rva(0x44E952); // scene::end_scene
pub const SHOW_FRAME: usize = rva(0x4057D1); // gfx::show_frame

/// A step: the function, and whether a set carry after it ends the frame.
pub type Step = (usize, bool);

/// The steps in order (every register kept, `pusha` .. `popa`): whether the frame was
/// made (the original's carry clear).
pub unsafe fn run_steps(entry: &Regs, steps: &[Step], env: &mut dyn Env) -> bool {
    let mut g = *entry;
    for &(function, checked) in steps {
        env.call(function, &mut g);
        if checked && g.carry() {
            return false;
        }
    }
    true
}

/// The test: the copy of `address` (`len` bytes, `entry` its first instructions' bytes,
/// `calls` their offsets) against `run_steps`.
pub fn test_steps(
    mut t: SelfTest,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    steps: &[Step],
) -> (usize, usize) {
    let places: Vec<(usize, u32)> = steps
        .iter()
        .enumerate()
        .map(|(n, &(f, _))| (f, n as u32))
        .collect();
    let mut known: Vec<(usize, u32)> = Vec::new();
    for &(f, p) in &places {
        if !known.iter().any(|&(k, _)| k == f) {
            known.push((f, p));
        }
    }
    let redirect = |target: usize| {
        known
            .iter()
            .find(|&&(function, _)| function == target)
            .map(|&(_, place)| frame_test::stand_in(place))
    };
    let copy =
        match unsafe { RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &redirect) }
        {
            Ok(copy) => copy,
            Err(why) => return t.skip(why),
        };
    let before = Regs {
        ebx: 0x1234,
        ecx: 7,
        ..Regs::SENTINEL
    };
    for failing in (0..steps.len() as u32).chain([u32::MAX]) {
        frame_test::begin(failing, 0);
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (frame_test::notes(), !ro.carry());
        frame_test::begin(failing, 0);
        let made = unsafe { run_steps(&before, steps, &mut Noting { places: &known }) };
        let mine = (frame_test::notes(), made);
        let preserved = ro.preserved_except(&before, 0);
        t.check(theirs == mine && preserved, || {
            format!(
                "failing call {failing:#x}: preserved={preserved} original made={} calls={}; rust made={} calls={}",
                theirs.1,
                theirs.0.len(),
                mine.1,
                mine.0.len()
            )
        });
    }
    t.finish()
}
