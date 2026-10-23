//! The service area's boxes (`screens::draw_service_boxes` and its four frames): flat
//! see-through quads (`scene::draw_tinted_quad`) laid out by long tables of the original's
//! code. The hooks' bodies are written out from the disassembly by a translator (each quad
//! the original's six stores and its call; each `cmp` / `jz` / `jnz` / `jmp` a test and a
//! `break` out of a labelled block, every jump forward); this file holds what they share.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

/// A call by place: 1 `draw_tinted_quad`, 2 .. the function's other calls.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A variable by its address in IDA.
pub fn rd(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

pub fn set(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) };
}

pub fn addr(a: usize) -> u32 {
    game_addr(rva(a)) as u32
}

/// The dword at `p` (the original's `[esi]`).
pub fn at_esi(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

const QUAD_IN: usize = 0x623DE8; // dword_623DE8: colour, see-through; x at +0x28 / +0x2C, y at +0x38 / +0x3C

/// One quad: x .. x + w, y .. y + h, the colour and see-through, then `draw_tinted_quad`
/// (eax the see-through going in, as the original's last `mov`).
#[allow(clippy::too_many_arguments)] // the original's six stores and its call, one to one
pub fn quad(
    g: &mut Regs,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    colour: u32,
    see: u32,
    part: &mut Part,
) -> Result<(), Refused> {
    set(QUAD_IN + 0x28, x);
    set(QUAD_IN + 0x38, y);
    set(QUAD_IN + 0x2C, x.wrapping_add(w));
    set(QUAD_IN + 0x3C, y.wrapping_add(h));
    set(QUAD_IN, colour);
    set(QUAD_IN + 4, see);
    g.eax = see;
    part(1, g)
}

/// In the game: `draw_tinted_quad`'s Rust version for place 1.
pub fn quad_part(r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::draw_tinted_quad;
    *r = unsafe { draw_tinted_quad::draw_tinted_quad(r) }?;
    Ok(())
}
