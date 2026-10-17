//! `physics::feel_ground` (0x43F1A8): the work car's ground this step:
//! `physics::note_surface_mode`, `physics::probe_ground`, `physics::lowest_body_point`;
//! then which wheels are off the ground: `dword_71A220` 1 when the first two (the qword
//! at 0x71A5D8) are both 0, `dword_71A224` the same for the last two (0x71A5E0),
//! `dword_71A22C` 1 when all four are; `dword_71A228` the driven ones' (the first pair's,
//! or all four's with `g_carDriveKind` not 0), and `dword_71A1C0` 0 when they are not off;
//! `dword_6141DC` the word at +0x26 of the face under probe point 4 (`dword_71ADD8`). A
//! reading: the wheels in the air, the ground's kind.
//!
//! In:  nothing
//! Out: eax the face's word, ebx the face, esi its address; ecx, edx as `probe_ground` /
//!      `lowest_body_point` leave them; no flags read
//!
//! It begins with a call: `.private_copy` (its three calls relocated) with
//! `.manual_patch`.
//! Callers: `sub_43B635` (0x43B6F6, 0x43B860, 0x43B8FB), `sub_45D737`, `sub_45D7B6` (twice),
//! `sub_45E3BF`, `sub_45EFF3`.
//! Refuses (before anything) more than 0x40 probe points; `probe_ground`'s refusals
//! mid-way (it puts back what it wrote; `dword_71A6C8` is the original's to write again).
//! Self-test: in a race only: a copy of the original (its calls the game's) against the
//! Rust version on the game's own state with the work car's switches by lot; all put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write,
    hook_asm, rva,
};
use crate::physics::ground::Ground;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::feel_ground", rva(0x43F1A8), thunk, &ORIGINAL)
    .private_copy(0xCA)
    .relocate_calls(&[0, 5, 0xA])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn feel_ground(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if g(0x71A238) > 0x40 {
            return Err(BadInput("more than 0x40 probe points").into());
        }
        crate::physics::note_surface_mode::note_surface_mode();
        crate::physics::probe_ground::probe_ground(r)?;
        crate::physics::lowest_body_point::lowest_body_point(r)?;
        let front = g(0x71A5D8) | g(0x71A5DC);
        let back = g(0x71A5E0) | g(0x71A5E4);
        p(0x71A220, u32::from(front == 0));
        p(0x71A224, u32::from(back == 0));
        p(0x71A22C, u32::from((front | back) == 0));
        r.eax = if g(0x71A018) != 0 {
            g(0x71A22C)
        } else {
            g(0x71A220)
        };
        p(0x71A228, r.eax);
        if r.eax == 0 {
            p(0x71A1C0, 0);
        }
        r.ebx = g(0x71ADD8);
        r.esi = (r.ebx << 6).wrapping_add(g(0x5CCCBC));
        r.eax = u32::from(((r.esi + 0x26) as *const u16).read_unaligned());
        p(0x6141DC, r.eax);
        Ok(())
    }
}

fn left_of(g: &Regs) -> Left {
    Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut gr = *r;
    unsafe { feel_ground(&mut gr) }?;
    Ok(left_of(&gr))
}

/// Everything it writes.
pub fn regions() -> Vec<Region> {
    let a = |x: u32| game_addr(rva(x as usize)) as u32;
    let mut r = crate::physics::probe_ground::regions(&Ground::game());
    r.extend([
        (a(0x71A6C8), 4),
        (a(0x6D3068), 12),
        (a(0x71A220), 0x10),
        (a(0x71A1C0), 4),
        (a(0x6141DC), 4),
    ]);
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() || g(0x71A238) == 0 || g(0x71A238) > 0x40 {
        return t.skip("not in a race (no stage, or no probe points)");
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F1A8,
            len: 0xCA,
            entry: &[0xE8, 0xD3, 0xFF, 0xFF, 0xFF],
            calls: &[0, 5, 0xA],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let inputs = [0x71A65C, 0x71A5D8, 0x71A5DC, 0x71A5E0, 0x71A5E4, 0x71A018];
    let mut all = regions();
    all.extend(
        inputs
            .iter()
            .map(|&v| (game_addr(rva(v as usize)) as u32, 4)),
    );
    let saved = capture(&all);
    for seed in 0..8u32 {
        p(0x71A65C, seed % 4);
        for (k, v) in [0x71A5D8u32, 0x71A5DC, 0x71A5E0, 0x71A5E4]
            .into_iter()
            .enumerate()
        {
            p(v, u32::from((seed >> k) & 1 == 1));
        }
        p(0x71A018, seed % 2);
        let input = capture(&all);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        restore(&all, &input);
        let mut gr = Regs::SENTINEL;
        let mine = unsafe { feel_ground(&mut gr) };
        let state = capture(&all);
        restore(&all, &saved);
        let regs_ok = mine.is_ok() && left_of(&gr).matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == state, || {
            let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
            format!("seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
        });
    }
    drop(copy);
    t.finish()
}
