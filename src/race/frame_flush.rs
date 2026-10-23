//! The race frame's device batches and the view's backdrop under `sub_45429B` (part R3, IDA
//! 2026-10-18): two batches of triangles flushed by the device (`sub_41ADF5`, the platform's
//! `DrawPrimitive`), and a quad over the view window.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

fn ga(a: u32) -> u32 {
    game_addr(rva(a as usize)) as u32
}

/// The render states both batches set: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (2, 0x14), (1, 0x1B)];

/// A batch: its count, its first word (into eax for the draw), the vertices (ebx) and
/// indices (ecx), where the count is kept, and the second render state's value (+0x14).
pub struct Batch {
    pub count: u32,
    pub word: u32,
    pub vertices: u32,
    pub indices: u32,
    pub kept: u32,
    pub blend: u32,
    pub text: u32,
}

/// `race::flush_triangle_batch`'s (0x45FDCF).
pub const PLAIN: Batch = Batch {
    count: 0x6E44F4,
    word: 0x6E44F8,
    vertices: 0x6D94F4,
    indices: 0x6E14F4,
    kept: 0x6EF7C4,
    blend: 2,
    text: 0x4DDCAC,
};

/// `race::flush_texture_batch`'s (0x45FEFE).
pub const TEXTURED: Batch = Batch {
    count: 0x6EF7BC,
    word: 0x6EF7C0,
    vertices: 0x6E44FC,
    indices: 0x6EC6FC,
    kept: 0x6EF7C8,
    blend: 6,
    text: 0x4DDCB4,
};

/// `race::flush_triangle_batch` (0x45FDCF) / `race::flush_texture_batch` (0x45FEFE): the
/// first sets texture `dword_5D1D04` (part 1, `scene::set_texture`); the render states
/// (part 2 each, eax the value, ebx the state); with triangles in the batch (its count also
/// into its kept count) the second looks up its texture (`scene::look_up_texture`) and sets
/// it (part 1) inside `pusha`; the draw (part 3: `sub_41ADF5`, edx the count, eax the word,
/// ebx the vertices, ecx the indices), a failure (eax not 0) logged as "drawtri" (part 4,
/// eax the text) and its code (part 5); the count and word 0. Gives the registers it leaves.
pub unsafe fn flush_batch(
    entry: &Regs,
    b: &Batch,
    textured: bool,
    part: crate::race::frame::Part,
) -> Result<Regs, Refused> {
    unsafe {
        let mut r = *entry;
        if !textured {
            r.eax = k(0x5D1D04);
            part(1, &mut r)?;
        }
        for (value, state) in STATES {
            r.eax = value;
            r.ebx = state;
            if state == 0x14 {
                r.eax = b.blend;
            }
            part(2, &mut r)?;
        }
        r.edx = k(b.count);
        w(b.kept, r.edx);
        if r.edx != 0 {
            r.ecx = ga(b.indices);
            r.ebx = ga(b.vertices);
            r.eax = k(b.word);
            let mut d = r;
            if textured {
                // `pusha`; eax `dword_6EDF5C` (which the look-up does not read).
                let mut t = r;
                crate::scene::look_up_texture::look_up_texture();
                t.eax = k(0x6240D4);
                part(1, &mut t)?;
            }
            part(3, &mut d)?;
            if d.eax != 0 {
                let mut log = Regs {
                    eax: ga(b.text),
                    ..d
                };
                part(4, &mut log)?;
                let mut log = d;
                part(5, &mut log)?;
            }
            // The second's draw is inside `pusha` .. `popa`.
            if !textured {
                r = d;
            }
        }
        w(b.count, 0);
        w(b.word, 0);
        Ok(r)
    }
}

/// The batches' parts in the game: 1 `scene::set_texture`, 2 `scene::set_render_state`,
/// 3 the device (`sub_41ADF5`), 4 / 5 the log's text and code (`sub_401427`, `sub_401622`).
pub fn batch_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::set_texture::set_texture_in_game(r),
            2 => {
                let (v, s) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, v, s)
            }
            p => {
                let at = [0x41ADF5usize, 0x401427, 0x401622][p as usize - 3];
                crate::calls::call(rva(at), r);
            }
        }
    }
    Ok(())
}

/// `fild i; fadd/fsub one; fstp`.
fn int_and_one(i: u32, add: bool) -> u32 {
    let mut out = 0u32;
    let v = [i, unsafe { game_read::<u32>(rva(0x5C5D38)) }];
    unsafe {
        if add {
            core::arch::asm!(
                "fild dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]",
                v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        } else {
            core::arch::asm!(
                "fild dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]",
                v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }
    out
}

/// `fild i; fdiv a; fdiv b; fstp`.
fn int_over_two(i: u32, a: u32, b: u32) -> u32 {
    let (v, mut out) = ([i, a, b], 0u32);
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
            "fstp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `race::draw_view_backdrop` (0x4651E6): a quad over the view window (`dword_623FB0` ..,
/// one pixel wider each way; its corners kept in `dword_623E10` .. `dword_623E24`) with
/// texture place (0xF80000, 0x80000) / `flt_5C5E2C` / `flt_5C5D98` at every corner (kept in
/// `dword_623DF0` / `dword_623E00`), colour 0xFF000000, `g_polygonVertexCount` 4; drawn with
/// the glow's render states (part 1, `sub_44E44D`), texture `dword_5D1DE8` (part 2,
/// `scene::set_texture`), the device's unclipped fan (part 3, `d3d::draw_fan_unclipped`),
/// then the depth buffer on (part 4).
pub unsafe fn draw_view_backdrop(r: &mut Regs) -> Result<(), Refused> {
    w(0x623DF0, 0xF8_0000);
    w(0x623E00, 0x8_0000);
    let left = k(0x623FB0);
    w(0x623E10, left);
    w(0x623E14, left.wrapping_add(k(0x62408C)));
    let top = k(0x623FB4);
    w(0x623E20, top);
    r.eax = top.wrapping_add(k(0x624090));
    w(0x623E24, r.eax);
    let v = |o: u32| 0x9605A0 + o;
    let x0 = int_and_one(k(0x623E10), false);
    w(v(0), x0);
    w(v(0x60), x0);
    let x1 = int_and_one(k(0x623E14), true);
    w(v(0x20), x1);
    w(v(0x40), x1);
    let y0 = int_and_one(k(0x623E20), false);
    w(v(4), y0);
    w(v(0x24), y0);
    let y1 = int_and_one(k(0x623E24), true);
    w(v(0x44), y1);
    w(v(0x64), y1);
    let (a, b) = (k(0x5C5E2C), k(0x5C5D98));
    for n in 0..4 {
        w(v(0x18 + 0x20 * n), int_over_two(k(0x623DF0), a, b));
    }
    for n in 0..4 {
        w(v(0x1C + 0x20 * n), int_over_two(k(0x623E00), a, b));
    }
    r.eax = 0xFF00_0000;
    for n in 0..4 {
        w(v(0x10 + 0x20 * n), r.eax);
    }
    w(0x6240F8, 4);
    crate::race::frame_flush::backdrop_part(1, r)?;
    r.eax = k(0x5D1DE8);
    crate::race::frame_flush::backdrop_part(2, r)?;
    crate::race::frame_flush::backdrop_part(3, r)?;
    crate::race::frame_flush::backdrop_part(4, r)
}

/// Its parts in the game.
pub fn backdrop_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::glow_render_states::glow_render_states(r),
            2 => crate::scene::set_texture::set_texture_in_game(r),
            3 => crate::calls::call(rva(0x41AD9C), r),
            _ => crate::scene::depth_buffer_on::depth_buffer(r, true),
        }
    }
    Ok(())
}
