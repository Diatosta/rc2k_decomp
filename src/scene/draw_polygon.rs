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
//! The x87 and `rcpss` are used with the original's own instructions. Refuses a polygon of no
//! vertices (the original then goes through 2^32) or of more than 256, a list entry past the
//! work table and an entry without its vertex.

use crate::d3d::cards_store::CARDS_3D;
use crate::d3d::draw_fan_unclipped::VERTICES_RVA;
use crate::d3d::{NO_TEXTURE, VERTEX_LEN};
use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::frame_store::FRAME_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use crate::scene::submit_polygon::{self, submit_polygon};

// ---- game data ----

pub const ENTRIES_RVA: usize = rva(0x975510); // dword_975510
pub const ENTRY_LEN: usize = 0x44;
pub const ENTRIES: u32 = 0x80;
pub const LIST_RVA: usize = rva(0x977710); // dword_977710
pub const MOST: u32 = 0x100;
pub const SWITCH_B_RVA: usize = rva(0x6240E4); // dword_6240E4
const TEXTURE_A_RVA: usize = rva(0x6240BC); // dword_6240BC
const TEXTURE_B_RVA: usize = rva(0x6240C0); // dword_6240C0
const SMOOTH_A_RVA: usize = rva(0x63E168); // dword_63E168
const SMOOTH_B_RVA: usize = rva(0x63E16C); // dword_63E16C
const SMOOTH_RVA: usize = rva(0x63E164); // dword_63E164
const TEXTURE_OF_RVA: usize = rva(0x5D1AE8); // dword_5D1AE8
const OTHER_TEXTURE_OF_RVA: usize = rva(0x5D3AE8); // dword_5D3AE8
const TEXTURES_LOADED_RVA: usize = rva(0x5DC970); // dword_5DC970

const MIRRORED_RVA: usize = rva(0x63993C); // dword_63993C
const W_FACTOR_RVA: usize = rva(0x6D180C); // flt_6D180C
const SSE_RVA: usize = rva(0x621A58); // dword_621A58

const FADED_RVA: usize = rva(0x63660C); // dword_63660C
const FADE_ALPHA_RVA: usize = rva(0x606284); // dword_606284
const FADE_TEXTURE: u32 = 0xC0;
const HIT_WORK_RVA: usize = rva(0x637648); // dword_637648 .. dword_63765C
const CLEARS_SWITCH_RVA: usize = rva(0x621904); // dword_621904
const MATERIALS_RVA: usize = rva(0x979F30); // unk_979F30
pub const CHECKED_RVA: usize = rva(0x62400C); // dword_62400C

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
        if FRAME_WORK.sun_on_screen.bits() == 0 {
            return;
        }
        let point = crate::session::read_bits(&FRAME_WORK.sun_hit).map(|v| v as i32);
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
        FRAME_WORK.sun_on_screen.set_bits(0u32);
    }
}

/// `sub_44C9A9`, and `sub_44E6F4` for a mirrored view: the positions.
unsafe fn place(entries: &[usize], mirrored: bool) {
    unsafe {
        let (x_offset, y_offset): (u32, u32) = (
            DRAW.view_offset_float[0].bits(),
            DRAW.view_offset_float[1].bits(),
        );
        let depth_factor: u32 = DRAW.one_over_farthest.bits();
        let sse = game_read::<u32>(SSE_RVA) != 0;
        let mirror_at: u32 = DRAW.view_centre[0].bits();
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
                    x87::rcpss_times(dword(vertex + 0x18), NUMBERS.k65536.bits())
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
        let number: u32 = DRAW.texture_page.bits();
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
        DRAW.polygon_texture.set_bits(texture.unwrap_or(NO_TEXTURE));
    }
}

/// `sub_44B94F`: clears a switch by what the entries' bytes +0x21 hold.
unsafe fn clear_a_switch(entries: &[usize]) {
    unsafe {
        if DRAW.polygon_sides[0].bits() == 0 || game_read::<u32>(SWITCH_B_RVA) == 0 {
            return;
        }
        let bytes = entries
            .iter()
            .map(|&entry| ((entry + 0x21) as *const u8).read());
        let (all, any) = bytes.fold((0xFFu8, 0u8), |(all, any), b| (all & b, any | b));
        if all == 0xFF {
            let material = game_addr(MATERIALS_RVA) + RACE_VIEW.texture_b.bits() as usize * 0x40;
            if ((material + 0x2E) as *const u8).read() == 0 {
                DRAW.polygon_sides[0].set_bits(0u32);
            }
        } else if any == 0 {
            DRAW.polygon_sides[1].set_bits(0u32);
        }
    }
}

unsafe fn shade(entries: &[usize], how: Shade) {
    unsafe {
        let colour: u32 = DRAW.polygon_colour.bits();
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
                    put(entry + 0x24, DRAW.polygon_alpha.bits() << 24 | colour);
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
        let count: u32 = DRAW.polygon_vertex_count.bits();
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of too many"));
        }
        let mut entries = entries(count)?;
        // Nothing has been written up to here.
        hit_test(&entries);
        let kept: [u32; 3] = [
            DRAW.polygon_sides[0].bits(),
            game_read(SWITCH_B_RVA),
            RACE_VIEW.polygon_mode.bits(),
        ];
        crate::session::write_bits(&DRAW.polygon_kept, kept);
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
            let (mode, texture): (u32, u32) =
                (RACE_VIEW.polygon_mode.bits(), DRAW.texture_page.bits());
            // `sub_44CAD5`.
            let (from, alpha): (u32, u32) = (CARDS_3D.fade_from.get(), game_read(FADE_ALPHA_RVA));
            let uv = crate::session::read_bits(&RACE_VARS.fade_uv);
            for (n, &entry) in entries.iter().enumerate() {
                let out = game_addr(VERTICES_RVA) + n * VERTEX_LEN;
                let by = ((entry + 0x20) as *const u8).read() as u32;
                put(out + 0x10, faded(from, 0x10, by) & 0xFF_FFFF | alpha << 24);
                put(out + 0x14, 0xFF00_0000);
                put(out + 0x18, uv[0]);
                put(out + 0x1C, uv[1]);
            }
            DRAW.texture_page.set_bits(FADE_TEXTURE);
            RACE_VIEW.polygon_mode.set_bits(7u32);
            choose_texture(outside);
            for (state, value) in [(0x15, 4), (0x13, 5), (0x14, 6), (0x1B, 1), (0x1C, 0)] {
                outside.state(state, value);
            }
            submit_polygon(outside);
            DRAW.texture_page.set_bits(texture);
            RACE_VIEW.polygon_mode.set_bits(mode);
        }

        // The mode, with its fallbacks.
        let mut mode: u32 = RACE_VIEW.polygon_mode.bits();
        if mode == 3 && game_read::<u32>(CLEARS_SWITCH_RVA) != 0 {
            clear_a_switch(&entries);
        }
        if matches!(mode, 3..=5) {
            let fallback = if game_read::<u32>(SWITCH_B_RVA) == 0 {
                Some(2)
            } else if DRAW.polygon_sides[0].bits() == 0 {
                Some([8, 9, 0xA][mode as usize - 3])
            } else {
                None
            };
            if let Some(fallback) = fallback {
                mode = fallback;
                RACE_VIEW.polygon_mode.set_bits(mode);
            }
        }
        if let Some((how, passes)) = passes(mode) {
            shade(&entries, how);
            for pass in passes {
                let (texture, smooth) = match pass.set {
                    Set::A => (TEXTURE_A_RVA, SMOOTH_A_RVA),
                    Set::B => (TEXTURE_B_RVA, SMOOTH_B_RVA),
                };
                DRAW.texture_page.set_bits(game_read::<u32>(texture));
                game_write(SMOOTH_RVA, game_read::<u32>(smooth));
                copy_set(&entries, pass.set);
                if let Some(as_mode) = pass.as_mode {
                    RACE_VIEW.polygon_mode.set_bits(as_mode);
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
                    RACE_VIEW.polygon_mode.set_bits(mode);
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
        DRAW.polygon_sides[0].set_bits(kept[0]);
        DRAW.polygon_sides[1].set_bits(kept[1]);
        RACE_VIEW.polygon_mode.set_bits(kept[2]);
        Ok(())
    }
}

/// The game's Direct3D layer and log.
pub struct Game;

impl submit_polygon::Outside for Game {
    fn set_texture(&mut self, texture: u32) {
        submit_polygon::Game.set_texture(texture)
    }
    fn draw_fan(&mut self) -> u32 {
        submit_polygon::Game.draw_fan()
    }
    fn bad_vertex(&mut self) {
        submit_polygon::Game.bad_vertex()
    }
    fn not_drawn(&mut self, result: u32) {
        submit_polygon::Game.not_drawn(result)
    }
}

impl Outside for Game {
    fn state(&mut self, state: u32, value: u32) {
        crate::d3d::set_render_state::set_render_state(value, state);
    }
}

/// `draw_polygon` with the game's Direct3D layer (`scene::submit_polygon`'s), for Rust
/// callers.
pub unsafe fn draw_polygon_in_game() -> Result<(), BadInput> {
    unsafe { draw_polygon(&mut Game) }
}
