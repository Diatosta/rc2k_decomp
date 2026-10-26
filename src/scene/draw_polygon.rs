//! `scene::draw_polygon` (0x44BA2C): shades one polygon and draws it, in one or two
//! passes by its material.
//!
//! In:  nothing, all in variables (below)
//! Out: the original leaves every register changed; its eleven callers go on without
//!      reading them, and the Rust version keeps them
//!
//! The polygon is `g_polygonVertexCount` entries of the work table `dword_975510` (0x44
//! bytes each), picked through the list `dword_977710`. An entry: +4 the projected
//! vertex (its +0xC x, +0x1C y, +0x18 depth), +8 and +0x10 two pairs of texture
//! coordinates, +0x18 the vertex's light as a colour, +0x20 a byte (fog or alpha by
//! the pass) and +0x21 another, +0x24 and +0x2C two pairs of colour and specular that
//! this function fills.
//!
//! What it does, in order:
//! 1. `sub_450147`: while `dword_6415C4` is set and the polygon has three vertices or
//!    more, tests whether the point `dword_6415C8`, `dword_6415CC` lies inside it
//!    (each edge by a 64-bit cross product of whole numbers) and clears the switch if
//!    it does.
//! 2. `sub_44B9BE`: keeps the two material switches and the mode, put back at the end.
//! 3. With `dword_63993C` set (a mirrored view, by what it does): the list's entries
//!    after the first are reversed, and after 4 every x is mirrored.
//! 4. `sub_44C9A9`: the positions into the vertex buffer: x and y plus an offset, the
//!    depth times a factor, and 1/w as 0.01 / that, or with `dword_621A58` set (SSE)
//!    `rcpss` of the depth times 65536.
//! 5. With `dword_63660C` set: a first pass in a colour faded from `dword_5AB964`
//!    towards 0x10 by each entry's byte +0x20 (`sub_44A4C5`), with texture 0xC0.
//! 6. The pass or two of the mode `dword_63DAB4` (0, 2 .. 6, 8, 9, 0xA; anything else
//!    draws nothing): the entries' colours (`Shade`), which texture and coordinates,
//!    five or six render states, `scene::submit_polygon`. `PASSES` has the table.
//!    Modes 3, 4 and 5 fall back to 2, and to 8, 9 and 0xA, when one of the switches
//!    `dword_6240E0`, `dword_6240E4` is 0; mode 3 first lets `sub_44B94F` clear a
//!    switch by the entries' bytes +0x21.
//!
//! The x87 and `rcpss` are used with the original's own instructions.
//! Shadow mode: the Rust version with its drawing only worked out, everything it
//! wrote put back, the original with its calls into the Direct3D layer noted; the
//! variables and the two lists of calls compared.
//! Manual patch with a private copy (its first instruction is a call): 0xEE0 bytes,
//! 120 calls relocated.
//! Refuses a polygon of no vertices (the original then goes through 2^32) or of more
//! than 256, a list entry past the work table and an entry without its vertex.

use crate::d3d::draw_fan_unclipped::{COUNT_RVA, VERTICES_RVA};
use crate::d3d::recorder::{Call, record};
use crate::d3d::set_texture::CURRENT_RVA;
use crate::d3d::{NO_TEXTURE, VERTEX_LEN};
use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::scene::submit_polygon::{self, submit_polygon};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::draw_polygon", rva(0x44BA2C), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xEE0;
const CALLS: [usize; 120] = [
    0x0, 0x5, 0x13, 0x18, 0x26, 0x44, 0x5D, 0x6C, 0x7B, 0x8A, 0x99, 0xA8, 0xAD, 0x182, 0x187,
    0x196, 0x1A5, 0x1B4, 0x1C3, 0x1C8, 0x1D7, 0x2D9, 0x2DE, 0x2ED, 0x2FC, 0x30B, 0x31A, 0x329,
    0x32E, 0x463, 0x472, 0x481, 0x490, 0x49F, 0x4AE, 0x4BD, 0x4C2, 0x4EF, 0x4F4, 0x503, 0x512,
    0x521, 0x530, 0x53F, 0x544, 0x5A5, 0x5AA, 0x5B9, 0x5C8, 0x5D7, 0x5E6, 0x5F5, 0x5FA, 0x714,
    0x719, 0x728, 0x737, 0x746, 0x755, 0x764, 0x769, 0x882, 0x887, 0x896, 0x8A5, 0x8B4, 0x8C3,
    0x8D2, 0x8D7, 0x9FC, 0xA01, 0xA10, 0xA1F, 0xA2E, 0xA3D, 0xA4C, 0xA51, 0xA60, 0xA6F, 0xA7E,
    0xA8D, 0xA9C, 0xAAF, 0xBD4, 0xBE3, 0xBF2, 0xC01, 0xC10, 0xC1F, 0xC2E, 0xC33, 0xC60, 0xC65,
    0xC6A, 0xC79, 0xC88, 0xC97, 0xCA6, 0xCB5, 0xCBA, 0xCBF, 0xDE8, 0xDF7, 0xE06, 0xE15, 0xE24,
    0xE33, 0xE42, 0xE47, 0xE74, 0xE79, 0xE7E, 0xE8D, 0xE9C, 0xEAB, 0xEBA, 0xEC9, 0xECE, 0xED3,
    0xEDA,
];

// ---- game data ----

pub const ENTRIES_RVA: usize = rva(0x975510); // dword_975510
pub const ENTRY_LEN: usize = 0x44;
pub const ENTRIES: u32 = 0x80;
pub const LIST_RVA: usize = rva(0x977710); // dword_977710
pub const MOST: u32 = 0x100;

const MODE_RVA: usize = rva(0x63DAB4); // dword_63DAB4
pub const SWITCH_A_RVA: usize = rva(0x6240E0); // dword_6240E0
pub const SWITCH_B_RVA: usize = rva(0x6240E4); // dword_6240E4
const KEPT_RVA: usize = rva(0x6240E8); // dword_6240E8, dword_6240EC, dword_6240F0
const COLOUR_RVA: usize = rva(0x6240CC); // dword_6240CC
const ALPHA_RVA: usize = rva(0x6240D0); // dword_6240D0
const TEXTURE_A_RVA: usize = rva(0x6240BC); // dword_6240BC
const TEXTURE_B_RVA: usize = rva(0x6240C0); // dword_6240C0
const TEXTURE_RVA: usize = rva(0x6240C4); // dword_6240C4
const SMOOTH_A_RVA: usize = rva(0x63E168); // dword_63E168
const SMOOTH_B_RVA: usize = rva(0x63E16C); // dword_63E16C
const SMOOTH_RVA: usize = rva(0x63E164); // dword_63E164
const POLYGON_TEXTURE_RVA: usize = rva(0x6240D4); // g_polygonTexture
const TEXTURE_OF_RVA: usize = rva(0x5D1AE8); // dword_5D1AE8
const OTHER_TEXTURE_OF_RVA: usize = rva(0x5D3AE8); // dword_5D3AE8
const TEXTURES_LOADED_RVA: usize = rva(0x5DC970); // dword_5DC970

const MIRRORED_RVA: usize = rva(0x63993C); // dword_63993C
const MIRROR_AT_RVA: usize = rva(0x62409C); // flt_62409C
const X_OFFSET_RVA: usize = rva(0x6241F4); // flt_6241F4
const Y_OFFSET_RVA: usize = rva(0x6241F8); // flt_6241F8
const DEPTH_FACTOR_RVA: usize = rva(0x6241CC); // flt_6241CC
const W_FACTOR_RVA: usize = rva(0x6D180C); // flt_6D180C
const SSE_RVA: usize = rva(0x621A58); // dword_621A58
const SSE_W_FACTOR_RVA: usize = rva(0x5C5D98); // flt_5C5D98

const FADED_RVA: usize = rva(0x63660C); // dword_63660C
const FADE_FROM_RVA: usize = rva(0x5AB964); // dword_5AB964
const FADE_ALPHA_RVA: usize = rva(0x606284); // dword_606284
const FADE_UV_RVA: usize = rva(0x623DBC); // dword_623DBC, dword_623DC0
const FADE_TEXTURE: u32 = 0xC0;

const HIT_TEST_RVA: usize = rva(0x6415C4); // dword_6415C4
const HIT_POINT_RVA: usize = rva(0x6415C8); // dword_6415C8, dword_6415CC
const HIT_WORK_RVA: usize = rva(0x637648); // dword_637648 .. dword_63765C
const CLEARS_SWITCH_RVA: usize = rva(0x621904); // dword_621904
const MATERIAL_RVA: usize = rva(0x63DA38); // dword_63DA38
const MATERIALS_RVA: usize = rva(0x979F30); // unk_979F30
pub const CHECKED_RVA: usize = rva(0x62400C); // dword_62400C

const SET_RENDER_STATE: usize = 0x41AB5B; // d3d::set_render_state

// ---- what the modes do ----

/// How the entries' colours are filled before a mode's passes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shade {
    /// +0x24 the material's colour, +0x28 nothing.
    Plain,
    /// +0x24 the material's colour with its alpha, +0x28 nothing.
    WithAlpha,
    /// +0x24 the light times the colour, +0x28 the byte +0x20 as the specular's alpha.
    Lit,
    /// `Lit`, and +0x2C a grey of the two bytes' product, +0x30 opaque black.
    LitAndGrey,
    /// `Lit`, and +0x2C the first colour with the byte +0x21 as its alpha, +0x30 as
    /// +0x28 (`true`) or opaque black.
    LitAndSecond(bool),
}

/// Which of the two textures and sets of coordinates and colours a pass takes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Set {
    A,
    B,
}

/// One pass: the mode stored while its texture is looked up (if another than the
/// mode's own), the set, render state 0x0E cleared around it, the states set before
/// it, and those after it.
struct Pass {
    as_mode: Option<u32>,
    set: Set,
    no_depth: bool,
    states: &'static [(u32, u32)],
    after: &'static [(u32, u32)],
}

const BLENDED: &[(u32, u32)] = &[(0x15, 4), (0x1C, 1), (0x13, 5), (0x14, 6), (0x1B, 1)];
const ADDED: &[(u32, u32)] = &[(0x15, 4), (0x1C, 1), (0x13, 5), (0x14, 2), (0x1B, 1)];
const GLOSS: &[(u32, u32)] = &[(0x15, 2), (0x13, 1), (0x14, 4), (0x1C, 0), (0x1B, 1)];

const fn pass(set: Set, states: &'static [(u32, u32)]) -> Pass {
    Pass {
        as_mode: None,
        set,
        no_depth: false,
        states,
        after: &[],
    }
}

/// The shading and the passes of a mode, once its fallbacks are taken.
fn passes(mode: u32) -> Option<(Shade, Vec<Pass>)> {
    let first_of_two = |mode: u32| Pass {
        as_mode: Some(mode),
        ..pass(Set::A, BLENDED)
    };
    let second = |states| Pass {
        no_depth: true,
        ..pass(Set::B, states)
    };
    Some(match mode {
        0 => (
            Shade::Plain,
            vec![Pass {
                states: &[(0x15, 2), (0x1B, 0), (0x1C, 0), (0x0F, 0)],
                after: &[(0x0F, 1)],
                ..pass(Set::A, &[])
            }],
        ),
        2 => (Shade::Lit, vec![pass(Set::A, BLENDED)]),
        3 => (
            Shade::LitAndSecond(true),
            vec![first_of_two(2), second(BLENDED)],
        ),
        4 => (
            Shade::LitAndSecond(false),
            vec![first_of_two(2), second(ADDED)],
        ),
        5 => (
            Shade::LitAndGrey,
            vec![first_of_two(2), pass(Set::B, GLOSS)],
        ),
        6 => (
            Shade::WithAlpha,
            vec![pass(
                Set::A,
                &[(0x15, 4), (0x1B, 1), (0x13, 5), (0x14, 6), (0x1C, 0)],
            )],
        ),
        8 => (Shade::LitAndSecond(true), vec![pass(Set::B, BLENDED)]),
        9 => (Shade::LitAndSecond(false), vec![pass(Set::B, ADDED)]),
        0xA => (
            Shade::LitAndGrey,
            vec![Pass {
                after: BLENDED,
                ..pass(Set::B, GLOSS)
            }],
        ),
        _ => return None,
    })
}

// ---- Rust version ----

/// What the function asks of the code around it.
pub trait Outside: submit_polygon::Outside {
    /// `d3d::set_render_state`.
    fn state(&mut self, state: u32, value: u32);
}

unsafe fn dword(at: usize) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) }
}

/// The three low bytes of `a` each times the same byte of `b`, over 256.
fn times(a: u32, b: u32) -> u32 {
    [0, 8, 16].into_iter().fold(0, |out, at| {
        out | (((a >> at & 0xFF) * (b >> at & 0xFF)) >> 8) << at
    })
}

/// `sub_44A4C5`: the three low bytes of `from` each moved towards the same byte of
/// `to` by `by` over 256, held to a byte; the top byte is `from`'s.
fn faded(from: u32, to: u32, by: u32) -> u32 {
    [0, 8, 16].into_iter().fold(from, |out, at| {
        let (a, b) = ((from >> at & 0xFF) as i32, (to >> at & 0xFF) as i32);
        let moved = ((b - a).wrapping_mul(by as i32) >> 8) + a;
        out & !(0xFF << at) | (moved.clamp(0, 0xFF) as u32) << at
    })
}

/// The polygon's entries, in the list's order, as addresses.
unsafe fn entries(count: u32) -> Result<Vec<usize>, BadInput> {
    (0..count as usize)
        .map(|n| unsafe {
            let number: u32 = game_read(LIST_RVA + n * 4);
            if number >= ENTRIES {
                return Err(BadInput("a list entry past the work table"));
            }
            let entry = game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN;
            if dword(entry + 4) == 0 {
                return Err(BadInput("an entry without its vertex"));
            }
            Ok(entry)
        })
        .collect()
}

/// `sub_450147` with `sub_4501B6`: the hit test.
unsafe fn hit_test(entries: &[usize]) {
    unsafe {
        if game_read::<u32>(HIT_TEST_RVA) == 0 {
            return;
        }
        let point: [i32; 2] = game_read(HIT_POINT_RVA);
        game_write(HIT_WORK_RVA, point);
        if entries.len() < 3 {
            return;
        }
        for n in 0..entries.len() {
            let corner = |entry: usize| {
                let vertex = dword(entry + 4) as usize;
                [
                    x87::to_int(dword(vertex + 0x0C)),
                    x87::to_int(dword(vertex + 0x1C)),
                ]
            };
            let (from, to) = (corner(entries[n]), corner(entries[(n + 1) % entries.len()]));
            game_write(HIT_WORK_RVA + 8, [from, to]);
            let along = point[1].wrapping_sub(from[1]) as i64 * to[0].wrapping_sub(from[0]) as i64;
            let across = point[0].wrapping_sub(from[0]) as i64 * to[1].wrapping_sub(from[1]) as i64;
            // Not on the inner side of this edge: the switch stays.
            if across.wrapping_sub(along) >= 0 {
                return;
            }
        }
        game_write(HIT_TEST_RVA, 0u32);
    }
}

/// `sub_44C9A9`, and `sub_44E6F4` for a mirrored view: the positions.
unsafe fn place(entries: &[usize], mirrored: bool) {
    unsafe {
        let (x_offset, y_offset): (u32, u32) = (game_read(X_OFFSET_RVA), game_read(Y_OFFSET_RVA));
        let depth_factor: u32 = game_read(DEPTH_FACTOR_RVA);
        let sse = game_read::<u32>(SSE_RVA) != 0;
        let mirror_at: u32 = game_read(MIRROR_AT_RVA);
        for (n, &entry) in entries.iter().enumerate() {
            let vertex = dword(entry + 4) as usize;
            let out = game_addr(VERTICES_RVA) + n * VERTEX_LEN;
            let mut x = x87::add(dword(vertex + 0x0C), x_offset);
            let depth = x87::mul(dword(vertex + 0x18), depth_factor);
            put(out + 4, x87::add(dword(vertex + 0x1C), y_offset));
            put(out + 8, depth);
            put(
                out + 0x0C,
                if sse {
                    x87::rcpss_times(dword(vertex + 0x18), game_read(SSE_W_FACTOR_RVA))
                } else {
                    x87::reciprocal_times(depth, game_read(W_FACTOR_RVA))
                },
            );
            if mirrored && entries.len() > 2 {
                x = x87::mirrored(x, mirror_at, x_offset);
            }
            put(out, x);
        }
    }
}

/// `sub_44CA7A`: a set's texture coordinates, colour and specular into the buffer.
unsafe fn copy_set(entries: &[usize], set: Set) {
    let (uv, colour) = match set {
        Set::A => (0x08, 0x24),
        Set::B => (0x10, 0x2C),
    };
    for (n, &entry) in entries.iter().enumerate() {
        unsafe {
            let out = game_addr(VERTICES_RVA) + n * VERTEX_LEN;
            put(out + 0x18, x87::loaded(dword(entry + uv)));
            put(out + 0x1C, x87::loaded(dword(entry + uv + 4)));
            put(out + 0x10, dword(entry + colour));
            put(out + 0x14, dword(entry + colour + 4));
        }
    }
}

/// `sub_44CC8D` with `sub_44E668`: the texture filter's state, and the polygon's
/// texture from the number in `dword_6240C4`.
unsafe fn choose_texture(outside: &mut dyn Outside) {
    unsafe {
        let smooth = game_read::<u32>(SMOOTH_RVA) == 1;
        outside.state(3, if smooth { 3 } else { 1 });
        let number: u32 = game_read(TEXTURE_RVA);
        let loaded: u32 = game_read(TEXTURES_LOADED_RVA);
        let texture = match number {
            u32::MAX => None,
            0x8000_0000.. => {
                let number = number & 0x7FFF_FFFF;
                (number < 0xFA0).then(|| game_read(OTHER_TEXTURE_OF_RVA + number as usize * 4))
            }
            _ => (number < 0x400).then(|| game_read(TEXTURE_OF_RVA + number as usize * 4)),
        };
        let texture = texture.filter(|&texture: &u32| texture < loaded);
        game_write(POLYGON_TEXTURE_RVA, texture.unwrap_or(NO_TEXTURE));
    }
}

/// `sub_44B94F`: clears a switch by what the entries' bytes +0x21 hold.
unsafe fn clear_a_switch(entries: &[usize]) {
    unsafe {
        if game_read::<u32>(SWITCH_A_RVA) == 0 || game_read::<u32>(SWITCH_B_RVA) == 0 {
            return;
        }
        let bytes = entries
            .iter()
            .map(|&entry| ((entry + 0x21) as *const u8).read());
        let (all, any) = bytes.fold((0xFFu8, 0u8), |(all, any), b| (all & b, any | b));
        if all == 0xFF {
            let material =
                game_addr(MATERIALS_RVA) + game_read::<u32>(MATERIAL_RVA) as usize * 0x40;
            if ((material + 0x2E) as *const u8).read() == 0 {
                game_write(SWITCH_A_RVA, 0u32);
            }
        } else if any == 0 {
            game_write(SWITCH_B_RVA, 0u32);
        }
    }
}

unsafe fn shade(entries: &[usize], how: Shade) {
    unsafe {
        let colour: u32 = game_read(COLOUR_RVA);
        for &entry in entries {
            let (fog, other) = (
                ((entry + 0x20) as *const u8).read() as u32,
                ((entry + 0x21) as *const u8).read() as u32,
            );
            match how {
                Shade::Plain => {
                    put(entry + 0x24, colour);
                    put(entry + 0x28, 0);
                    continue;
                }
                Shade::WithAlpha => {
                    put(entry + 0x24, game_read::<u32>(ALPHA_RVA) << 24 | colour);
                    put(entry + 0x28, 0);
                    continue;
                }
                _ => {}
            }
            let lit = times(dword(entry + 0x18), colour) | 0xFF00_0000;
            put(entry + 0x24, lit);
            put(entry + 0x28, fog << 24);
            match how {
                Shade::LitAndGrey => {
                    put(entry + 0x2C, ((other * fog) >> 8) * 0x01_0101);
                    put(entry + 0x30, 0xFF00_0000);
                }
                Shade::LitAndSecond(fogged) => {
                    put(entry + 0x2C, lit & 0xFF_FFFF | other << 24);
                    put(entry + 0x30, if fogged { fog << 24 } else { 0xFF00_0000 });
                }
                _ => {}
            }
        }
    }
}

pub unsafe fn draw_polygon(outside: &mut dyn Outside) -> Result<(), BadInput> {
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of too many"));
        }
        let mut entries = entries(count)?;
        // Nothing has been written up to here.
        hit_test(&entries);
        let kept: [u32; 3] = [
            game_read(SWITCH_A_RVA),
            game_read(SWITCH_B_RVA),
            game_read(MODE_RVA),
        ];
        game_write(KEPT_RVA, kept);
        let mirrored = game_read::<u32>(MIRRORED_RVA) != 0;
        if mirrored && count > 2 {
            // `sub_44E6B9`: the list's entries after the first, turned round.
            entries[1..].reverse();
            for (n, entry) in entries.iter().enumerate().skip(1) {
                let number = (entry - game_addr(ENTRIES_RVA)) / ENTRY_LEN;
                game_write(LIST_RVA + n * 4, number as u32);
            }
        }
        place(&entries, mirrored);

        if game_read::<u32>(FADED_RVA) != 0 {
            let (mode, texture): (u32, u32) = (game_read(MODE_RVA), game_read(TEXTURE_RVA));
            // `sub_44CAD5`.
            let (from, alpha): (u32, u32) = (game_read(FADE_FROM_RVA), game_read(FADE_ALPHA_RVA));
            let uv: [u32; 2] = game_read(FADE_UV_RVA);
            for (n, &entry) in entries.iter().enumerate() {
                let out = game_addr(VERTICES_RVA) + n * VERTEX_LEN;
                let by = ((entry + 0x20) as *const u8).read() as u32;
                put(out + 0x10, faded(from, 0x10, by) & 0xFF_FFFF | alpha << 24);
                put(out + 0x14, 0xFF00_0000);
                put(out + 0x18, uv[0]);
                put(out + 0x1C, uv[1]);
            }
            game_write(TEXTURE_RVA, FADE_TEXTURE);
            game_write(MODE_RVA, 7u32);
            choose_texture(outside);
            for (state, value) in [(0x15, 4), (0x13, 5), (0x14, 6), (0x1B, 1), (0x1C, 0)] {
                outside.state(state, value);
            }
            submit_polygon(outside);
            game_write(TEXTURE_RVA, texture);
            game_write(MODE_RVA, mode);
        }

        // The mode, with its fallbacks.
        let mut mode: u32 = game_read(MODE_RVA);
        if mode == 3 && game_read::<u32>(CLEARS_SWITCH_RVA) != 0 {
            clear_a_switch(&entries);
        }
        if matches!(mode, 3..=5) {
            let fallback = if game_read::<u32>(SWITCH_B_RVA) == 0 {
                Some(2)
            } else if game_read::<u32>(SWITCH_A_RVA) == 0 {
                Some([8, 9, 0xA][mode as usize - 3])
            } else {
                None
            };
            if let Some(fallback) = fallback {
                mode = fallback;
                game_write(MODE_RVA, mode);
            }
        }
        if let Some((how, passes)) = passes(mode) {
            shade(&entries, how);
            for pass in passes {
                let (texture, smooth) = match pass.set {
                    Set::A => (TEXTURE_A_RVA, SMOOTH_A_RVA),
                    Set::B => (TEXTURE_B_RVA, SMOOTH_B_RVA),
                };
                game_write(TEXTURE_RVA, game_read::<u32>(texture));
                game_write(SMOOTH_RVA, game_read::<u32>(smooth));
                copy_set(&entries, pass.set);
                if let Some(as_mode) = pass.as_mode {
                    game_write(MODE_RVA, as_mode);
                }
                if pass.no_depth {
                    outside.state(0x0E, 0);
                }
                choose_texture(outside);
                for &(state, value) in pass.states {
                    outside.state(state, value);
                }
                submit_polygon(outside);
                if pass.as_mode.is_some() {
                    game_write(MODE_RVA, mode);
                }
                if pass.no_depth {
                    outside.state(0x0E, 1);
                }
                for &(state, value) in pass.after {
                    outside.state(state, value);
                }
            }
        }
        // `sub_44B9DD`.
        game_write(SWITCH_A_RVA, kept[0]);
        game_write(SWITCH_B_RVA, kept[1]);
        game_write(MODE_RVA, kept[2]);
        Ok(())
    }
}

/// The game's Direct3D layer and log.
pub struct Game<'a>(pub submit_polygon::Game<'a>);

impl submit_polygon::Outside for Game<'_> {
    fn set_texture(&mut self, texture: u32) {
        self.0.set_texture(texture)
    }
    fn draw_fan(&mut self) -> u32 {
        self.0.draw_fan()
    }
    fn bad_vertex(&mut self) {
        self.0.bad_vertex()
    }
    fn not_drawn(&mut self, result: u32) {
        self.0.not_drawn(result)
    }
}

impl Outside for Game<'_> {
    fn state(&mut self, state: u32, value: u32) {
        let mut r = Regs {
            eax: value,
            ebx: state,
            ..*self.0.0
        };
        crate::calls::call(rva(SET_RENDER_STATE), &mut r);
    }
}

/// Works out what would be drawn without drawing it.
#[derive(Default)]
struct Foreseen(submit_polygon::Foreseen);

impl submit_polygon::Outside for Foreseen {
    fn set_texture(&mut self, texture: u32) {
        self.0.set_texture(texture);
    }
    fn draw_fan(&mut self) -> u32 {
        self.0.draw_fan()
    }
    fn bad_vertex(&mut self) {
        self.0.bad_vertex()
    }
    fn not_drawn(&mut self, result: u32) {
        self.0.not_drawn(result)
    }
}

impl Outside for Foreseen {
    fn state(&mut self, state: u32, value: u32) {
        self.0.calls.push(Call::State { state, value });
    }
}

// ---- hook ----

/// Everything the function and what it calls may write, for a polygon of `count`.
fn regions(count: u32) -> Vec<Region> {
    let count = count.min(MOST) as usize;
    let mut regions = vec![
        (game_addr(VERTICES_RVA) as u32, count * VERTEX_LEN),
        (game_addr(ENTRIES_RVA) as u32, ENTRIES as usize * ENTRY_LEN),
        (game_addr(LIST_RVA) as u32, (count + 1) * 4),
        (game_addr(KEPT_RVA) as u32, 12),
        (game_addr(HIT_WORK_RVA) as u32, 24),
    ];
    regions.extend(
        [
            MODE_RVA,
            SWITCH_A_RVA,
            SWITCH_B_RVA,
            TEXTURE_RVA,
            SMOOTH_RVA,
            POLYGON_TEXTURE_RVA,
            HIT_TEST_RVA,
            CURRENT_RVA,
            CHECKED_RVA,
        ]
        .map(at),
    );
    regions
}

/// `regions` with only the polygon's own entries instead of the whole work table: the
/// self-test compares the whole table, shadow mode what a polygon can reach.
fn shadow_regions(count: u32) -> Vec<Region> {
    let mut regions = regions(count);
    if let Ok(entries) = unsafe { entries(count.min(MOST)) } {
        regions.remove(1);
        regions.extend(entries.into_iter().map(|entry| (entry as u32, ENTRY_LEN)));
    }
    regions
}

/// `draw_polygon` with the game's Direct3D layer (`scene::submit_polygon`'s), the
/// registers at the call passed on, for Rust callers.
pub unsafe fn draw_polygon_in_game(entry: &Regs) -> Result<(), BadInput> {
    unsafe { draw_polygon(&mut Game(submit_polygon::Game(entry))) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        let entry = *r;
        let done = unsafe { draw_polygon_in_game(&entry) };
        if let Err(e) = done {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let (count, mode): (u32, u32) = unsafe { (game_read(COUNT_RVA), game_read(MODE_RVA)) };
    let regions = shadow_regions(count);
    let before = capture(&regions);
    let mut foreseen = Foreseen::default();
    let mine = unsafe { draw_polygon(&mut foreseen) }.map(|()| capture(&regions));
    restore(&regions, &before);
    let (_, theirs_calls) = record(false, || unsafe { call_original(r) });
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    if mine != theirs || foreseen.0.calls != theirs_calls {
        HOOK.mismatch(|| {
            format!(
                "{count} vertices, mode {mode:#x}: first different byte {:?} of {}; original's calls {theirs_calls:x?}; rust's {:x?}",
                mine.iter().zip(&theirs).position(|(a, b)| a != b),
                mine.len(),
                foreseen.0.calls
            )
        });
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let done = unsafe { draw_polygon_in_game(&entry) };
    if let Err(e) = done {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test: polygons of the test's in the game's own tables, nothing drawn ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (width, height): (u32, u32) =
        unsafe { (game_read(rva(0x4F494C)), game_read(rva(0x4F4950))) };
    if width < 400 || height < 300 {
        return t.skip("there is no screen mode");
    }
    const COUNT: u32 = 4;
    let regions = regions(COUNT);
    // What the test sets besides: the inputs.
    let inputs: Vec<Region> = [
        COUNT_RVA,
        COLOUR_RVA,
        ALPHA_RVA,
        TEXTURE_A_RVA,
        TEXTURE_B_RVA,
        SMOOTH_A_RVA,
        SMOOTH_B_RVA,
        MIRRORED_RVA,
        MIRROR_AT_RVA,
        X_OFFSET_RVA,
        Y_OFFSET_RVA,
        DEPTH_FACTOR_RVA,
        SSE_RVA,
        FADED_RVA,
        FADE_FROM_RVA,
        FADE_ALPHA_RVA,
        FADE_UV_RVA,
        FADE_UV_RVA + 4,
        HIT_POINT_RVA,
        HIT_POINT_RVA + 4,
        CLEARS_SWITCH_RVA,
        rva(0x5183A8), // g_hasMmx
    ]
    .map(at)
    .to_vec();
    let saved = (capture(&regions), capture(&inputs));
    let f = |value: f32| value.to_bits();
    // The projected vertices: memory of the test's, laid out as the function reads it.
    let mut projected = [[0u32; 8]; COUNT as usize];
    let corners = [
        (20.0f32, 30.0f32, 0.25f32),
        (300.5, 40.0, 0.5),
        (280.0, 200.25, 0.75),
        (10.0, 180.0, 0.125),
    ];
    for (vertex, (x, y, depth)) in projected.iter_mut().zip(corners) {
        (vertex[3], vertex[7], vertex[6]) = (f(x), f(y), f(depth));
    }
    let lights = [0x00FF_FFFFu32, 0x0080_4020, 0x1234_5678, 0];
    let bytes = [(0xFFu8, 0xFFu8), (0x80, 0x40), (0x00, 0xFF), (0x33, 0x00)];
    let mut cases = 0u32;
    for mode in [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB] {
        for (switch_a, switch_b) in [(1u32, 1u32), (0, 1), (1, 0)] {
            // The variants are gone through in turn, so that each meets every mode.
            cases += 1;
            let variant = cases % 6;
            let (mirrored, sse, fade, hit, mmx, all_ff) = (
                variant == 1,
                variant == 2,
                variant == 3,
                variant == 4,
                variant != 5,
                variant == 0,
            );
            let run = |rust: bool| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                for n in 0..COUNT as usize {
                    // The list picks the entries out of order.
                    let number = [5u32, 2, 9, 0][n];
                    game_write(LIST_RVA + n * 4, number);
                    let entry = game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN;
                    put(entry + 4, projected[n].as_ptr() as u32);
                    for (k, value) in [0.1f32, 0.9, 0.3, 0.7].into_iter().enumerate() {
                        put(entry + 8 + k * 4, f(value + n as f32));
                    }
                    put(entry + 0x18, lights[n]);
                    let (fog, other) = if all_ff { (0xFF, 0xFF) } else { bytes[n] };
                    ((entry + 0x20) as *mut u8).write(fog);
                    ((entry + 0x21) as *mut u8).write(other);
                }
                game_write(COUNT_RVA, COUNT);
                game_write(MODE_RVA, mode);
                game_write(SWITCH_A_RVA, switch_a);
                game_write(SWITCH_B_RVA, switch_b);
                game_write(COLOUR_RVA, 0x00C0_80FFu32);
                game_write(ALPHA_RVA, 0x7Fu32);
                game_write(TEXTURE_A_RVA, 3u32);
                game_write(TEXTURE_B_RVA, 0x8000_0002u32);
                game_write(TEXTURE_RVA, u32::MAX);
                game_write(SMOOTH_A_RVA, 1u32);
                game_write(SMOOTH_B_RVA, 0u32);
                game_write(SMOOTH_RVA, 0u32);
                game_write(MIRRORED_RVA, mirrored as u32);
                game_write(MIRROR_AT_RVA, f(160.0));
                game_write(X_OFFSET_RVA, f(0.5));
                game_write(Y_OFFSET_RVA, f(0.25));
                game_write(DEPTH_FACTOR_RVA, f(1.0));
                game_write(SSE_RVA, sse as u32);
                game_write(FADED_RVA, fade as u32);
                game_write(FADE_FROM_RVA, 0x55A0_C0E0u32);
                game_write(FADE_ALPHA_RVA, 0x60u32);
                game_write(FADE_UV_RVA, [f(0.5), f(0.5)]);
                game_write(HIT_TEST_RVA, hit as u32);
                game_write(HIT_POINT_RVA, [100i32, 100]);
                game_write(CLEARS_SWITCH_RVA, variant % 2);
                game_write(rva(0x5183A8), mmx as u32);
                game_write(CURRENT_RVA, 7u32);
                if rust {
                    let mut foreseen = Foreseen::default();
                    let done = draw_polygon(&mut foreseen).is_ok();
                    (done, foreseen.0.calls, capture(&regions))
                } else {
                    let mut ro = Regs::SENTINEL;
                    let (_, calls) = record(true, || call_original(&mut ro));
                    (true, calls, capture(&regions))
                }
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && !theirs.1.is_empty() == passes_drawn(mode, fade), || {
                format!(
                    "mode {mode:#x}, switches {switch_a} {switch_b}, variant {variant}: original's calls {:x?}; rust done {} calls {:x?}; first different byte {:?}",
                    theirs.1,
                    mine.0,
                    mine.1,
                    theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
                )
            });
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}

/// Whether a mode makes any call into the Direct3D layer at all: so that a test whose
/// two sides both drew nothing where something was to be drawn does not pass.
fn passes_drawn(mode: u32, fade: bool) -> bool {
    fade || passes(mode).is_some()
}
