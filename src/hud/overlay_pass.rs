//! `hud::overlay_pass` (0x45A893): the race frame's 2D pass: `dword_6D14F2` the players
//! (`dword_6238A4`); `scene::depth_buffer_off`; `hud::overlay_states`; with `dword_63DB98`
//! 2, a player whose car block's +0xFA8 is set (`dword_710394` / `dword_711374`) keeps only
//! bit 0x40 of its HUD flags (`dword_6D14F6` / `dword_6D14FA`, put back after) and its
//! `unk_6D1486` / `unk_6D1496` is 0; `hud::draw_hud`; `scene::depth_buffer_on`;
//! `scene::full_view_window`; render state 3 = 3; `dword_63E168` and `dword_63E16C` 1;
//! `hud::draw_view_strips`; an empty function (`nullsub_9`); `scene::depth_buffer_off`;
//! `scene::set_filters`.
//!
//! In:  nothing
//! Out: as the last call leaves them
//!
//! Caller: `sub_4547AE` (0x454786). The window is its Rust version; the rest through
//! `part` (1 / 4 the depth buffer off / on, 2 the states, 3 the HUD, 5 the render state,
//! 6 the strips, 7 the filters), in the game their Rust versions. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose calls go to stand-ins (the window's
//! to its hook), against the Rust version, modes and blocks by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::overlay_pass", rva(0x45A893), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4
const PLAYERS_COPY_RVA: usize = rva(0x6D14F2); // dword_6D14F2
const FLAGS_RVA: [usize; 2] = [rva(0x6D14F6), rva(0x6D14FA)]; // the HUD flags
const MODE_RVA: usize = rva(0x63DB98); // dword_63DB98
const BLOCK_RVA: [usize; 2] = [rva(0x710394), rva(0x711374)]; // the car blocks' +0xFA8
const CLEARED_RVA: [usize; 2] = [rva(0x6D1486), rva(0x6D1496)];
const ON_RVA: [usize; 2] = [rva(0x63E168), rva(0x63E16C)];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn overlay_pass(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(PLAYERS_RVA);
        game_write(PLAYERS_COPY_RVA, g.eax);
        part(1, &mut g)?;
        part(2, &mut g)?;
        let kept: [u32; 2] = FLAGS_RVA.map(|r| game_read(r));
        if game_read::<u32>(MODE_RVA) == 2 {
            for n in 0..2 {
                if game_read::<u32>(BLOCK_RVA[n]) != 0 {
                    game_write(FLAGS_RVA[n], game_read::<u32>(FLAGS_RVA[n]) & 0x40);
                    game_write(CLEARED_RVA[n], 0u32);
                }
            }
        }
        let done = part(3, &mut g);
        game_write(FLAGS_RVA[1], kept[1]);
        game_write(FLAGS_RVA[0], kept[0]);
        done?;
        part(4, &mut g)?;
        crate::scene::full_view_window::full_view_window().apply(&mut g);
        [g.eax, g.ebx] = [3, 3];
        part(5, &mut g)?;
        game_write(ON_RVA[0], 1u32);
        game_write(ON_RVA[1], 1u32);
        part(6, &mut g)?;
        part(1, &mut g)?;
        part(7, &mut g)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        match place {
            1 => crate::scene::depth_buffer_on::depth_buffer(r, false),
            2 => *r = overlay_states::overlay_states(r, &mut overlay_states::game_part)?,
            3 => *r = draw_hud::draw_hud(r, &mut draw_hud::game_part)?,
            4 => crate::scene::depth_buffer_on::depth_buffer(r, true),
            5 => overlay_states::game_part(place, r)?,
            6 => *r = draw_view_strips::draw_view_strips(r, &mut draw_view_strips::game_part)?,
            _ => crate::scene::set_filters::set_filters(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { overlay_pass(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, dword_6238A4
    let entry: &[u8] = &[0xA1, 0xA4, 0x38, 0x62, 0x00];
    let copy = match copy(
        0x45A893,
        0xB0,
        entry,
        &[0xA, 0xF, 0x5D, 0x6E, 0x73, 0x82, 0x9B, 0xA0, 0xA5, 0xAA],
        &[
            (0x44E3F1, 1),
            (0x45A943, 2),
            (0x4B1CD0, 3),
            (0x44E41F, 4),
            (0x44EB72, 5),
            (0x45B7B1, 6),
            (0x44E8FB, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut regions: Vec<Region> = [
        PLAYERS_COPY_RVA,
        FLAGS_RVA[0],
        FLAGS_RVA[1],
        CLEARED_RVA[0],
        CLEARED_RVA[1],
        ON_RVA[0],
        ON_RVA[1],
    ]
    .map(at)
    .to_vec();
    regions.extend(crate::scene::choose_view_window::regions());
    regions.push(at(crate::scene::choose_view_window::MODE_RVA));
    let mut vars = regions.clone();
    vars.extend([PLAYERS_RVA, MODE_RVA, BLOCK_RVA[0], BLOCK_RVA[1]].map(at));
    vars.extend(crate::scene::set_view_window::inputs());
    let saved = capture(&vars);
    for case in 0..5u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(PLAYERS_RVA, case & 1);
                game_write(MODE_RVA, if case >= 2 { 2u32 } else { case });
                game_write(BLOCK_RVA[0], u32::from(case == 2 || case == 4));
                game_write(BLOCK_RVA[1], u32::from(case >= 3));
                game_write(FLAGS_RVA[0], 0xFFu32);
                game_write(FLAGS_RVA[1], 0x7Eu32);
                game_write(CLEARED_RVA[0], 5u32);
                game_write(CLEARED_RVA[1], 6u32);
            },
            &|r| unsafe { overlay_pass(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
