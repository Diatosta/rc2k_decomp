//! `race::draw_mirror_view` (0x454806): the mirror's view, unless `dword_64BB18` or
//! `dword_71C3E0` is set or `dword_621480` (the mirror on, a reading) is 0:
//! `dword_63DC58` kept in `dword_63E170`; `dword_606294` and `g_gridReach` saved, made
//! `flt_5C5E2C`'s value (eax too) and 2; the mirror's window (`race::mirror_view_window`),
//! `dword_63DC58` back from `dword_63E170`, the backdrop (`race::draw_view_backdrop`), the
//! scene (`race::draw_view_scene`); the two put back; the full view's window again
//! (`race::full_view_window_2`).
//!
//! In:  nothing
//! Out: the registers as the last call leaves them; no flags read
//!
//! Callers: the views of `sub_45429B` (five). Its calls through `part` by place
//! (`CALLEES`); in the game `draw_view_scene`'s Rust version, the others (all hooked)
//! through the game. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose four calls go to stand-ins (which change the saved two, to see them put back),
//! against the Rust version, each switch; the calls, their registers and the variables
//! compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::race::mud::{copy, run_both};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_mirror_view", rva(0x454806), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 4] = [
    0x455F17, // 0 race::mirror_view_window
    0x4651E6, // 1 race::draw_view_backdrop
    0x4557AB, // 2 race::draw_view_scene
    0x456004, // 3 race::full_view_window_2
];

const SWITCHES: [usize; 3] = [rva(0x64BB18), rva(0x71C3E0), rva(0x621480)];
const VIEW_VALUE_RVA: usize = rva(0x63DC58); // dword_63DC58
const VIEW_VALUE_KEPT_RVA: usize = rva(0x63E170); // dword_63E170
const FAR_RVA: usize = rva(0x606294); // dword_606294
const MIRROR_FAR_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C
const GRID_REACH_RVA: usize = rva(0x60A87C); // g_gridReach

/// A call by place (`CALLEES`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_mirror_view(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        let g = |a: usize| game_read::<u32>(a);
        if g(SWITCHES[0]) != 0 || g(SWITCHES[1]) != 0 || g(SWITCHES[2]) == 0 {
            return Ok(r);
        }
        game_write(VIEW_VALUE_KEPT_RVA, g(VIEW_VALUE_RVA));
        let saved = (g(FAR_RVA), g(GRID_REACH_RVA));
        r.eax = g(MIRROR_FAR_RVA);
        game_write(FAR_RVA, r.eax);
        game_write(GRID_REACH_RVA, 2u32);
        let drawn = (|| {
            part(0, &mut r)?;
            game_write(VIEW_VALUE_RVA, g(VIEW_VALUE_KEPT_RVA));
            part(1, &mut r)?;
            part(2, &mut r)
        })();
        game_write(GRID_REACH_RVA, saved.1);
        game_write(FAR_RVA, saved.0);
        drawn?;
        part(3, &mut r)?;
    }
    Ok(r)
}

/// In the game: `draw_view_scene`'s Rust version, the others' through the game (their
/// hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 2 {
        return unsafe { crate::race::draw_view_scene::draw_view_scene_in_game(r) };
    }
    crate::calls::call(rva(CALLEES[place as usize]), r);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_mirror_view(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The stand-ins: noted (`level_init::note`), and the mirror's window changing the view's
/// value, the far value and the grid's reach, as a callee might.
fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    unsafe {
        game_write(VIEW_VALUE_RVA, 0x1111_0000 + place);
        game_write(FAR_RVA, 0x2222_0000 + place);
        game_write(GRID_REACH_RVA, 0x10 + place);
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_64BB18, 0
    let copy = match copy(
        0x454806,
        0x74,
        &[0x83, 0x3D, 0x18, 0xBB, 0x64, 0x00, 0x00],
        &[0x47, 0x58, 0x5D, 0x6E],
        &[(0x455F17, 0), (0x4651E6, 1), (0x4557AB, 2), (0x456004, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let regions: Vec<Region> = [
        &SWITCHES.map(at)[..],
        &[
            at(VIEW_VALUE_RVA),
            at(VIEW_VALUE_KEPT_RVA),
            at(FAR_RVA),
            at(MIRROR_FAR_RVA),
            at(GRID_REACH_RVA),
        ],
    ]
    .concat();
    let saved = capture(&regions);
    for switches in [[0u32, 0, 1], [1, 0, 1], [0, 1, 1], [0, 0, 0], [0, 0, 7]] {
        run_both(
            &mut t,
            &format!("switches {switches:x?}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&regions, &saved);
                for (at, v) in SWITCHES.into_iter().zip(switches) {
                    game_write(at, v);
                }
                game_write(VIEW_VALUE_RVA, 0xAAAA_0001u32);
                game_write(FAR_RVA, 0xAAAA_0002u32);
                game_write(GRID_REACH_RVA, 9u32);
            },
            &|r| unsafe { draw_mirror_view(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
