//! `marks::begin_quads` (0x4C4453): the render set up for the marks' quads and a quad's
//! four vertices made ready. `scene::set_texture` with `dword_5D1E10`,
//! `scene::depth_buffer_on`, `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`); then each of the four
//! `g_polygonVertices` gets +0xC `dword_91FFA6`, +0x10 -1, +0x14 0, and its texture place
//! from `dword_91FFAA` .. `dword_91FFB6` (u: AA, B2, B2, AA; v: AE, AE, B6, B6);
//! `g_polygonVertexCount` 4, `flt_8EB96D` 1 / `flt_6241C8`, `dword_91FFBA` 0.
//!
//! In:  nothing
//! Out: eax .. edx the four texture places, edi `g_polygonVertices`; esi and ebp as the
//!      render calls leave them
//!
//! Caller: `sub_4C3EEE`. Its calls through `part` (places 1 .. 4: the texture, the depth
//! buffer, the depth test, a render state); in the game their Rust versions (the
//! device's calls through their hooks). Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose eight calls go to stand-ins, against the Rust version
//! on the same stand-ins; the vertices, the globals and the calls compared.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::begin_quads", rva(0x4C4453), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: 0x20 bytes each
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const COLOUR_RVA: usize = rva(0x91FFA6); // dword_91FFA6
const PLACES_RVA: usize = rva(0x91FFAA); // dword_91FFAA .. dword_91FFB6
const SCALE_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const CLEARED_RVA: usize = rva(0x91FFBA); // dword_91FFBA
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// A call by place: 1 the texture, 2 the depth buffer on, 3 the depth test off, 4 a render
/// state (eax the value, ebx the state).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn begin_quads(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(TEXTURE_RVA);
        part(1, &mut g)?;
        part(2, &mut g)?;
        part(3, &mut g)?;
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            part(4, &mut g)?;
        }
        g.edi = game_addr(VERTICES_RVA) as u32;
        let colour: u32 = game_read(COLOUR_RVA);
        let [a, b, c, d]: [u32; 4] = game_read(PLACES_RVA);
        let uv = [(a, b), (c, b), (c, d), (a, d)];
        for (k, (u, v)) in uv.into_iter().enumerate() {
            let at = VERTICES_RVA + 0x20 * k;
            game_write(at + 0xC, colour);
            game_write(at + 0x10, 0xFFFF_FFFFu32);
            game_write(at + 0x14, 0u32);
            game_write(at + 0x18, u);
            game_write(at + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [a, b, c, d];
        game_write(COUNT_RVA, 4u32);
        game_write(SCALE_RVA, crate::math::x87::reciprocal(game_read(OF_RVA)));
        game_write(CLEARED_RVA, 0u32);
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::set_texture::set_texture_in_game(r),
            2 => crate::scene::depth_buffer_on::depth_buffer(r, true),
            3 => crate::scene::depth_test_off::depth_test_off(r),
            _ => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
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
    match unsafe { begin_quads(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { begin_quads(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}

pub fn regions() -> Vec<Region> {
    let mut r = vec![(game_addr(VERTICES_RVA) as u32, 0x80)];
    r.extend([COUNT_RVA, SCALE_RVA, CLEARED_RVA].map(at));
    r
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, dword_5D1E10
    let entry: &[u8] = &[0xA1, 0x10, 0x1E, 0x5D, 0x00];
    let copy = match copy(
        0x4C4453,
        0x136,
        entry,
        &[0x5, 0xA, 0xF, 0x1E, 0x2D, 0x3C, 0x4B, 0x5A],
        &[(0x44E77E, 1), (0x44E41F, 2), (0x44E3D1, 3), (0x44EB72, 4)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([TEXTURE_RVA, COLOUR_RVA, OF_RVA].map(at));
    vars.push((game_addr(PLACES_RVA) as u32, 16));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(TEXTURE_RVA, 0x40 + case);
                game_write(COLOUR_RVA, 0x1122_3344u32.rotate_left(case));
                game_write(OF_RVA, (0.5f32 + case as f32).to_bits());
                for k in 0..4 {
                    game_write(
                        PLACES_RVA + 4 * k,
                        ((k + case as usize) as f32 * 0.25).to_bits(),
                    );
                }
            },
            &|r| unsafe { begin_quads(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
