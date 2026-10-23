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
//! Out: esi past the items, edi `g_polygonVertices`; the rest as the walk leaves them
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

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
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

/// A call by place (see the header), with its x87 values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

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

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_squares(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let old = x87::set_chop();
    let done = unsafe { run(entry, part) };
    x87::set_control(old);
    done
}

unsafe fn run(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        part(1, &mut g, &[])?;
        g.edi = game_addr(VERTICES_RVA) as u32;
        let [a, b, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [a, b, c, d];
        if game_read::<u32>(RUNNING_RVA) != 0 {
            g = squares_spawn(&g);
        }
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            g.esi = e;
            let get = |o: u32| ((e + o) as *const u32).read_unaligned();
            if get(0) != 1 {
                continue;
            }
            if game_read::<u32>(RUNNING_RVA) != 0 {
                g.edx = game_read(TICKS_RVA);
                let age = get(8).wrapping_add(g.edx);
                ((e + 8) as *mut u32).write_unaligned(age);
                if age as i32 >= LIFE {
                    (e as *mut u32).write_unaligned(0);
                    continue;
                }
            }
            g.edx = get(8);
            let x = x87::to_int(get(0x20)) as u32;
            let z = x87::to_int(get(0x24)) as u32;
            g.eax = (x >> 3) & 0xFF;
            g.ebx = (((z >> 3) & 0x7F) << 8) + g.eax;
            if game_read::<u8>(EDGE_RVA + g.ebx as usize) != 0 {
                (e as *mut u32).write_unaligned(0);
                continue;
            }
            let alpha = x87::int_over_int_to_int(g.edx as i32, game_read(FADE_RVA)) as u32;
            g.ecx = game_read::<u32>(LIGHT_RVA) & 0xFF_FFFF;
            g.edx = ((!alpha) << 24) | g.ecx;
            let values = [less(game_read(FAR_RVA), get(0x24)), extended(get(0x20))];
            part(2, &mut g, &values)?;
            g.esi = e;
        }
        g.esi = base + ITEMS * ITEM_LEN;
        part(3, &mut g, &[])?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::{draw_square, squares_setup};
    unsafe {
        match place {
            1 => *r = squares_setup::squares_setup(r)?,
            2 => *r = draw_square::draw_square(r, [v[0], v[1]])?,
            _ => crate::scene::depth_buffer_on::depth_buffer(true),
        }
    }
    Ok(())
}
