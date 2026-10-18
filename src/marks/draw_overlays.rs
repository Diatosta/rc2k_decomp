//! `marks::draw_overlays` (0x4C74DA): the race frame's layer of marks drawn before the car's
//! shadow (a reading): `scene::depth_buffer_on`, `scene::depth_test_on`, render states
//! 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 1, 0x1B = 1; `marks::draw_plane`,
//! `marks::squares_frame`, `marks::hexagons_top`; then the second shadow, the twin of
//! `marks::draw_shadow` without its depth calls (`draw_shadow`'s `SECOND`).
//!
//! In:  nothing
//! Out: the registers as the path leaves them
//!
//! Caller: the race's frame (0x45632B, just before `draw_shadow`). Hooked through a
//! private copy (a call at the entry). The calls through `part` (1 the depth buffer, 2 the
//! depth test, 3 a render state, 4 `draw_plane`, 5 `squares_frame`, 6 `hexagons_top`, 7
//! `draw_fan`); the shadow's vector helpers their Rust versions. Refuses the shadow's
//! refusals (null settings, a corner past `g_marksCorners`, an angle `fsincos` cannot
//! take), before anything. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose calls go to stand-ins (the vector helpers real), against the Rust version,
//! as `draw_shadow`'s on the second shadow's variables.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use crate::marks::draw_shadow::{SECOND, SECOND_TEST, draw_shadow_with, test_prepare, test_vars};
use crate::marks::setup::SETTINGS_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::draw_overlays", rva(0x4C74DA), thunk, &ORIGINAL)
    .private_copy(0xACA)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLS: [usize; 30] = [
    0x0, 0x5, 0x14, 0x23, 0x32, 0x41, 0x50, 0x55, 0x5A, 0x5F, 0x7B, 0x8A, 0x99, 0xA8, 0xB7, 0x106,
    0x111, 0x128, 0x13F, 0x14A, 0x155, 0x1A1, 0x2FE, 0x315, 0x5DE, 0x709, 0x732, 0x75B, 0x784,
    0xAC4,
];
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The shadow's refusals, checked before anything is drawn.
fn shadow_refuses() -> Option<Refused> {
    let corner: u32 = unsafe { game_read(SECOND.corner()) };
    if corner == 0xFFFF_FFFF {
        return None;
    }
    if corner >= 0x100 {
        return Some(BadInput("a shadow corner past g_marksCorners").into());
    }
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Some(BadInput("no marks settings").into());
    }
    SECOND.angle_refuses()
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_overlays(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    if let Some(e) = shadow_refuses() {
        return Err(e);
    }
    part(1, &mut g)?;
    part(2, &mut g)?;
    for (value, state) in STATES {
        [g.eax, g.ebx] = [value, state];
        part(3, &mut g)?;
    }
    for place in [4, 5, 6] {
        part(place, &mut g)?;
    }
    unsafe {
        draw_shadow_with(&g, &SECOND, false, &mut |p, r| {
            part(if p == 4 { 7 } else { p }, r)
        })
    }
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::{draw_plane, draw_shadow, hexagons_top, squares_frame};
    unsafe {
        match place {
            1..=3 => draw_shadow::game_part(place, r)?,
            4 => *r = draw_plane::draw_plane(r, &mut draw_plane::game_part)?,
            5 => *r = squares_frame::squares_frame(r, &mut squares_frame::game_part)?,
            6 => *r = hexagons_top::hexagons_top(r, &mut hexagons_top::game_part)?,
            _ => draw_shadow::game_part(4, r)?,
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
    match unsafe { draw_overlays(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    use crate::service::harness::{capture, restore};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call scene::depth_buffer_on (its first bytes as the original's, not the hook's)
    let first: [u8; 5] = {
        let rel = (0x44E41Fi64 - (0x4C74DA + 5)) as i32;
        let b = rel.to_le_bytes();
        [0xE8, b[0], b[1], b[2], b[3]]
    };
    let copy = match copy(
        0x4C74DA,
        0xACA,
        &first,
        &CALLS,
        &[
            (0x44E41F, 1),
            (0x44E3E1, 2),
            (0x44EB72, 3),
            (0x4C8F64, 4),
            (0x4BF60F, 5),
            (0x4BAC87, 6),
            (0x44EA65, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2760 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let (regions, vars) = test_vars(&SECOND, &SECOND_TEST);
    let saved = capture(&vars);
    for case in 0..5u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                test_prepare(&SECOND, &SECOND_TEST, sa, case);
            },
            &|r| unsafe { draw_overlays(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
