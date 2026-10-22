//! `race::viewer_frame` (0x41D5FB): one frame of the replay viewer (`screens::run_frame`'s
//! function): at the replay's end (`dword_5BEC90`) the screen cleared
//! (`gfx::clear_back_surface`); the scene begun (`d3d::begin_scene`; carry: lost); unless
//! at the end, the block's cameras into it (`drive::replay_frame_marks`) and the frame
//! drawn (`race::show_replay_still`). A picture asked (`dword_5C2540`): asked no more,
//! the scene ended (`d3d::end_scene`), the picture saved (`race::viewer_snapshot`), shown.
//! Else, stopped (`dword_5BEC98` 1): the list's texture (`dword_5BECB4`,
//! `d3d::set_texture`), eight render states, each item (`race::draw_viewer_item`, ecx its
//! number, `[edi+1]` of them), the bar's quads (`drive::tyre_quad`: the frame, with blocks
//! the bar, the two marks and the place shown along it, the four buttons) and the panel
//! (`race::draw_viewer_panel`); the scene ended and the frame shown (`gfx::show_frame`).
//!
//! In:  nothing
//! Out: carry set when the frame was lost (the scene's begin or end, the showing); every
//!      register kept (`pusha` / `popa`; `screens::run_frame` reads only carry)
//!
//! The marks' and the place's positions are `mark * w / blocks` (`mul` / `div`): a quotient
//! past 32 bits (the original's divide error) is refused, before the bar is drawn. Caller:
//! `screens::run_frame` (eax, from `race::replay_viewer`). Its calls through `part`
//! (`CALLEES`); in the game their Rust versions, the scene's begin and end, the texture,
//! the render states and the showing through the game (the platform's). Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose calls go to stand-ins, against
//! the Rust version: at the end or not, a picture asked, playing or stopped, no blocks,
//! the begin, end or showing failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::race::viewer::{
    AT_END, BLOCK, BLOCKS, FRAME, MARK_IN, MARK_OUT, MENU, Part, SNAPSHOT, STOPPED, Saved, answer,
    dispatch_parts, dword, gb, region, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::viewer_frame", rva(0x41D5FB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 13] = [
    0, 0x40579A, // 1 gfx::clear_back_surface
    0x41ABC7, // 2 d3d::begin_scene (carry)
    0x41D8A3, // 3 drive::replay_frame_marks
    0x46062E, // 4 race::show_replay_still
    0x41AB8A, // 5 d3d::end_scene (carry)
    0x41EFAB, // 6 race::viewer_snapshot
    0x41ACF5, // 7 d3d::set_texture (eax)
    0x41AB5B, // 8 d3d::set_render_state (eax value, ebx state)
    0x41D94A, // 9 race::draw_viewer_item (esi, ecx)
    0x41DD3B, // 10 drive::tyre_quad (eax, ebx, ecx)
    0x41DA25, // 11 race::draw_viewer_panel
    0x4057D1, // 12 gfx::show_frame (carry)
];
const CALLS: [usize; 28] = [
    0xA, 0xF, 0x2E, 0x33, 0x4B, 0x56, 0x76, 0x85, 0x94, 0xA3, 0xB2, 0xC1, 0xD0, 0xDF, 0xEE, 0xFE,
    0x11D, 0x140, 0x175, 0x1AA, 0x1EA, 0x200, 0x20B, 0x221, 0x22C, 0x231, 0x236, 0x23D,
];

const LIST_TEXTURE: usize = 0x5BECB4; // dword_5BECB4
const BAR_X: usize = 0x5C29CC; // dword_5C29CC: the bar's rectangle (x, .., w at +8)
const BAR_W: usize = 0x5C29D4; // dword_5C29D4
const MARK_RECT: usize = 0x5C29DC; // dword_5C29DC
const PLACE_RECT: usize = 0x5C29EC; // dword_5C29EC

/// The Rust version: the registers it leaves (the entry's) and carry.
pub fn viewer_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut out = *entry;
    let lost = frame(entry, part)?;
    out.set_carry(lost);
    Ok(out)
}

/// Whether the frame was lost.
fn frame(entry: &Regs, part: Part) -> Result<bool, Refused> {
    let mut r = *entry;
    if g(AT_END) == 1 {
        part(1, &mut r)?;
    }
    part(2, &mut r)?;
    if r.carry() {
        return Ok(true);
    }
    if g(AT_END) != 1 {
        r.eax = g(BLOCK);
        r.ebx = g(FRAME);
        part(3, &mut r)?;
        part(4, &mut r)?;
    }
    if g(SNAPSHOT) == 1 {
        p(SNAPSHOT, 0);
        part(5, &mut r)?;
        if r.carry() {
            return Ok(true);
        }
        part(6, &mut r)?;
    } else {
        if g(STOPPED) == 1 {
            stopped(&mut r, part)?;
        }
        part(5, &mut r)?;
        if r.carry() {
            return Ok(true);
        }
    }
    part(12, &mut r)?;
    Ok(r.carry())
}

/// The list, the bar and the panel (stopped).
fn stopped(r: &mut Regs, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x}`.
    use rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x};
    r.eax = g(LIST_TEXTURE);
    part(7, r)?;
    for (value, state) in LIST_STATES {
        r.eax = value;
        r.ebx = state;
        part(8, r)?;
    }
    r.edi = a(MENU);
    r.esi = r.edi + 0x18;
    r.ecx = 0;
    loop {
        part(9, r)?;
        r.esi += 0x10;
        r.ecx = r.ecx.wrapping_add(1);
        if r.ecx as u8 == gb(MENU + 1) {
            break;
        }
    }
    let quad = |part: Part, r: &mut Regs, uv: usize, rect: usize, colour: u32| {
        r.eax = a(uv);
        r.ebx = a(rect);
        r.ecx = colour;
        part(10, r)
    };
    quad(part, r, 0x5C2C4C, 0x5C297C, 0xFF_FFFF)?;
    let blocks = g(BLOCKS);
    if blocks != 0 {
        let (w, x) = (g(BAR_W), g(BAR_X));
        let [first, second, here] =
            bar_places((g(MARK_IN), g(MARK_OUT)), g(BLOCK), g(FRAME), w, blocks)?;
        quad(part, r, 0x5C2C3C, 0x5C29CC, 0xFF00)?;
        // edx the division's remainder at each of the next calls.
        r.edx = first.1;
        p(MARK_RECT, mark_x(first.0, x));
        quad(part, r, 0x5C2ACC, MARK_RECT, 0xFF_FFFF)?;
        r.edx = second.1;
        p(MARK_RECT, mark_x(second.0, x));
        quad(part, r, 0x5C2ACC, MARK_RECT, 0xFF_FFFF)?;
        r.edx = here.1;
        p(PLACE_RECT, place_x(here.0, x));
        quad(part, r, 0x5C2AEC, PLACE_RECT, 0xFF_FFFF)?;
        quad(part, r, 0x5C2C2C, 0x5C298C, 0xFF_FFFF)?;
        quad(part, r, 0x5C2C2C, 0x5C29AC, 0xFF_FFFF)?;
        quad(part, r, 0x5C2C3C, 0x5C299C, 0xFF_FFFF)?;
        quad(part, r, 0x5C2C3C, 0x5C29BC, 0xFF_FFFF)?;
        part(11, r)?;
    }
    Ok(())
}

/// In the game: the Rust versions; the scene, the texture, the states and the showing
/// through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::*;
    unsafe {
        match place {
            1 => {
                *r = crate::gfx::clear_back_surface::clear_back_surface(
                    r,
                    &mut crate::gfx::clear_back_surface::game_part,
                )?
            }
            3 => crate::drive::replay_frame_marks::replay_frame_marks(),
            4 => *r = show_replay_still::show_replay_still(r, &mut show_replay_still::game_part)?,
            6 => *r = viewer_snapshot::viewer_snapshot(r, &mut viewer_snapshot::game_part)?,
            9 => *r = draw_viewer_item::draw_viewer_item(r, &mut draw_viewer_item::game_part)?,
            10 => {
                *r =
                    crate::drive::tyre_quad::tyre_quad(r, &mut crate::drive::tyre_parts::game_part)?
            }
            11 => *r = draw_viewer_panel::draw_viewer_panel(r, &mut draw_viewer_panel::game_part)?,
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        viewer_frame(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if gb(MENU + 1) == 0 {
        return t.skip("the viewer's list has no items");
    }
    let places: &'static [(usize, u32)] = Box::leak(
        (1..13u32)
            .map(|p| (CALLEES[p as usize], p))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // pusha; cmp dword_5BEC90, 1
    let copy = match crate::race::viewer::copy(
        0x41D5FB,
        0x24B,
        &[0x60, 0x83, 0x3D, 0x90, 0xEC, 0x5B, 0x00, 0x01],
        &CALLS,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [dword(SNAPSHOT), region(MARK_RECT, 4), region(PLACE_RECT, 4)];
    let mut all = regions.to_vec();
    all.extend([
        dword(AT_END),
        dword(STOPPED),
        dword(BLOCK),
        dword(BLOCKS),
        dword(FRAME),
        dword(MARK_IN),
        dword(MARK_OUT),
        dword(BAR_W),
        dword(BAR_X),
    ]);
    let saved = Saved::new(&all);
    // (what, at the end, snapshot, stopped, blocks, the call that fails)
    for (what, end, snapshot, stopped, blocks, fails) in [
        ("playing", 0u32, 0u32, 0u32, 9u32, 0u32),
        ("stopped", 0, 0, 1, 9, 0),
        ("stopped, no blocks", 0, 0, 1, 0, 0),
        ("at the end", 1, 0, 1, 9, 0),
        ("a picture", 0, 1, 1, 9, 0),
        ("the begin failing", 0, 0, 1, 9, 2),
        ("the end failing", 0, 0, 1, 9, 5),
        ("the showing failing", 0, 0, 1, 9, 12),
    ] {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            crate::race::viewer::CF,
            &|| {
                p(AT_END, end);
                p(SNAPSHOT, snapshot);
                p(STOPPED, stopped);
                p(BLOCKS, blocks);
                p(BLOCK, 4);
                p(FRAME, 0x31);
                p(MARK_IN, 2);
                p(MARK_OUT, 7);
                p(BAR_W, 0x200);
                p(BAR_X, 0x40);
                answer(Box::new(move |place, r| r.set_carry(place == fails)));
            },
            &|r| viewer_frame(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
