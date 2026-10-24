//! `marks::run_squares` (0x4BFFEC): the squares' frame, with the x87 rounding set to chop
//! throughout (put back at the end): `marks::squares_setup`; the vertices' texture places
//! from `dword_8BC222` ..; with `dword_7652A0` `marks::squares_spawn`; then each live square
//! item: with `dword_7652A0` its age (+8) plus `g_marksFrameTicks`, ended at 0xFA0 or more;
//! ended when its cell's byte in `g_marksEdgeMask` is set (the cell from its place, chopped,
//! shifted right 3); else drawn by `marks::draw_square` (st(1) its x, st(0) `flt_8BC236`
//! less its z; colour `g_viewLight`'s low 24 bits under the alpha: not (age over
//! `dword_8BC232`), shifted left 24). Last `scene::depth_buffer_on`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BF60F` (0x4BF7E9). The spawning is its Rust version; the set-up, the
//! drawing and the depth buffer through `part` (1 `squares_setup`, 2 `draw_square` with
//! its two values, 3 `depth_buffer_on`), in the game their Rust versions. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose three calls go to stand-ins
//! (the drawing's taking its values off), against the Rust version, on the game's items
//! (as `expire_squares`') with ages by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::marks::expire_squares::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::marks::squares_spawn::squares_spawn;
use crate::math::x87;
use crate::scene::matrix3::extended;

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: usize = rva(0x8BC222); // dword_8BC222 .. dword_8BC22E
const RUNNING_RVA: usize = rva(0x7652A0); // dword_7652A0
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const EDGE_RVA: usize = rva(0x809BC3); // g_marksEdgeMask
const FADE_RVA: usize = rva(0x8BC232); // dword_8BC232
const LIGHT_RVA: usize = rva(0x6415E4); // g_viewLight
const FAR_RVA: usize = rva(0x8BC236); // flt_8BC236
const LIFE: i32 = 0xFA0;

/// `fld k; fsub z`, kept extended.
fn less(k: u32, z: u32) -> [u8; 10] {
    let (v, mut out) = ([k, z], [0u8; 10]);
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fsub dword ptr [{v} + 4]",
            "fstp tbyte ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) out.as_mut_ptr(),
            out("st(0)") _,
            options(nostack),
        )
    };
    out
}

/// The Rust version (see the header).
pub unsafe fn run_squares() {
    let old = x87::set_chop();
    unsafe { run() };
    x87::set_control(old);
}

unsafe fn run() {
    unsafe {
        crate::marks::squares_setup::squares_setup();
        let [a, b, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        if game_read::<u32>(RUNNING_RVA) != 0 {
            squares_spawn();
        }
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            let get = |o: u32| ((e + o) as *const u32).read_unaligned();
            if get(0) != 1 {
                continue;
            }
            if game_read::<u32>(RUNNING_RVA) != 0 {
                let age = get(8).wrapping_add(game_read(TICKS_RVA));
                ((e + 8) as *mut u32).write_unaligned(age);
                if age as i32 >= LIFE {
                    (e as *mut u32).write_unaligned(0);
                    continue;
                }
            }
            let x = x87::to_int(get(0x20)) as u32;
            let z = x87::to_int(get(0x24)) as u32;
            let column = (x >> 3) & 0xFF;
            let cell = (((z >> 3) & 0x7F) << 8) + column;
            if game_read::<u8>(EDGE_RVA + cell as usize) != 0 {
                (e as *mut u32).write_unaligned(0);
                continue;
            }
            let alpha = x87::int_over_int_to_int(get(8) as i32, game_read(FADE_RVA)) as u32;
            let colour = ((!alpha) << 24) | (game_read::<u32>(LIGHT_RVA) & 0xFF_FFFF);
            let values = [less(game_read(FAR_RVA), get(0x24)), extended(get(0x20))];
            crate::marks::draw_square::draw_square(e, colour, values);
        }
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
}
