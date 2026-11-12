//! `scene::draw_polygon` (0x44BA2C): shades one polygon and draws it, in one or two
//! passes by its material.
//!
//! In:  nothing, all in variables (below)
//!
//! The polygon is `polygon_vertex_count` entries of the work table (0x44 bytes each),
//! picked through the entry list. An entry: +4 the projected vertex (its +0xC x, +0x1C y,
//! +0x18 depth), +8 and +0x10 two pairs of texture coordinates, +0x18 the vertex's light as
//! a colour, +0x20 a byte (fog or alpha by the pass) and +0x21 another, +0x24 and +0x2C two
//! pairs of colour and specular that this function fills.
//!
//! What it does, in order:
//! 1. While the sun's `on_screen` switch is set and the polygon has three vertices or
//!    more, tests whether the sun's hit point lies inside it (each edge by a 64-bit cross
//!    product of whole numbers) and clears the switch if it does.
//! 2. Keeps the two material switches and the mode, put back at the end.
//! 3. With `view_mirrored` set: the list's entries after the first are reversed, and after
//!    4 every x is mirrored.
//! 4. The positions into the vertex buffer: x and y plus an offset, the depth times a
//!    factor, and 1/w as 0.01 / that, or with SSE `rcpss` of the depth times 65536.
//! 5. With `faded` set: a first pass in a colour faded from `fade_from` towards 0x10 by each
//!    entry's byte +0x20 (`gfx::fade_colour`), with texture 0xC0.
//! 6. The pass or two of the polygon mode (0, 2 .. 6, 8, 9, 0xA; anything else draws
//!    nothing): the entries' colours (`Shade`), which texture and coordinates, five or six
//!    render states, `scene::submit_polygon`. `PASSES` has the table. Modes 3, 4 and 5 fall
//!    back to 2, and to 8, 9 and 0xA, when one of the two material switches is 0; mode 3
//!    first lets `clear_a_switch` clear a switch by the entries' bytes +0x21.
//!
//! The x87 and `rcpss` are used with the original's own rounding. Refuses a polygon of no
//! vertices (the original then goes through 2^32) or of more than 256, a list entry past the
//! work table and an entry without its vertex.

use crate::d3d::NO_TEXTURE;
use crate::d3d::vertices_store::PolygonVertices;
use crate::errors::BadInput;
use crate::math::x87;
use crate::race::owner::Race;
use crate::scene::polygon_store::PolygonEntry;
use crate::scene::submit_polygon::{self, submit_polygon};

// ---- game data ----

pub const ENTRY_LEN: usize = 0x44;
pub const ENTRIES: u32 = 0x80;
pub const MOST: u32 = 0x100;
const FADE_TEXTURE: u32 = 0xC0;

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
fn passes(mode: u32) -> Option<(Shade, smallvec::SmallVec<[Pass; 2]>)> {
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
            smallvec::smallvec![Pass {
                states: &[(0x15, 2), (0x1B, 0), (0x1C, 0), (0x0F, 0)],
                after: &[(0x0F, 1)],
                ..pass(Set::A, &[])
            }],
        ),
        2 => (Shade::Lit, smallvec::smallvec![pass(Set::A, BLENDED)]),
        3 => (
            Shade::LitAndSecond(true),
            smallvec::smallvec![first_of_two(2), second(BLENDED)],
        ),
        4 => (
            Shade::LitAndSecond(false),
            smallvec::smallvec![first_of_two(2), second(ADDED)],
        ),
        5 => (
            Shade::LitAndGrey,
            smallvec::smallvec![first_of_two(2), pass(Set::B, GLOSS)],
        ),
        6 => (
            Shade::WithAlpha,
            smallvec::smallvec![pass(
                Set::A,
                &[(0x15, 4), (0x1B, 1), (0x13, 5), (0x14, 6), (0x1C, 0)],
            )],
        ),
        8 => (
            Shade::LitAndSecond(true),
            smallvec::smallvec![pass(Set::B, BLENDED)],
        ),
        9 => (
            Shade::LitAndSecond(false),
            smallvec::smallvec![pass(Set::B, ADDED)],
        ),
        0xA => (
            Shade::LitAndGrey,
            smallvec::smallvec![Pass {
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

/// The vertex buffer's vertex `n` (the polygon has at most as many vertices: checked).
fn out(buffer: &PolygonVertices, n: usize) -> &crate::d3d::vertices_store::Vertex {
    &buffer.vertices[n]
}

/// An entry of the polygon: its number in the work table and the entry.
type Listed = (u32, &'static PolygonEntry);

/// The three low bytes of `a` each times the same byte of `b`, over 256.
fn times(a: u32, b: u32) -> u32 {
    [0, 8, 16].into_iter().fold(0, |out, at| {
        out | (((a >> at & 0xFF) * (b >> at & 0xFF)) >> 8) << at
    })
}

/// `gfx::fade_colour`: the three low bytes of `from` each moved towards the same byte of
/// `to` by `by` over 256, held to a byte; the top byte is `from`'s.
fn faded(from: u32, to: u32, by: u32) -> u32 {
    [0, 8, 16].into_iter().fold(from, |out, at| {
        let (a, b) = ((from >> at & 0xFF) as i32, (to >> at & 0xFF) as i32);
        let moved = ((b - a).wrapping_mul(by as i32) >> 8) + a;
        out & !(0xFF << at) | (moved.clamp(0, 0xFF) as u32) << at
    })
}

/// The polygon's entries, in the list's order.
fn entries(
    polygon_work: &'static crate::scene::polygon_store::PolygonWork,
    count: u32,
) -> Result<smallvec::SmallVec<[Listed; 32]>, BadInput> {
    (0..count as usize)
        .map(|n| {
            let number: u32 = crate::scene::polygon::list_cell(polygon_work, n).get();
            if number >= ENTRIES {
                return Err(BadInput("a list entry past the work table"));
            }
            let entry = crate::scene::polygon::entry(polygon_work, number as usize)?;
            if entry.record.get() == 0 {
                return Err(BadInput("an entry without its vertex"));
            }
            Ok((number, entry))
        })
        .collect()
}

/// The sun's hit test.
fn hit_test(
    part: &crate::workers::PartScratch,
    quad: &[crate::session::Dword; 32],
    sun: &crate::scene::polygon_work::ViewSun,
    entries: &[Listed],
) {
    if sun.on_screen.get() == 0 {
        return;
    }
    let point = crate::session::read_bits(&sun.hit).map(|v| v as i32);
    // (The original kept the point and each edge in `LightState.hit_work`, read by nothing.)
    if entries.len() < 3 {
        return;
    }
    for n in 0..entries.len() {
        let corner = |(_, entry): Listed| {
            let vertex = entry.record.get();
            [
                x87::to_int(crate::scene::point_records::vertex(
                    part, quad, vertex, 0x0C,
                )),
                x87::to_int(crate::scene::point_records::vertex(
                    part, quad, vertex, 0x1C,
                )),
            ]
        };
        let (from, to) = (corner(entries[n]), corner(entries[(n + 1) % entries.len()]));
        let along = point[1].wrapping_sub(from[1]) as i64 * to[0].wrapping_sub(from[0]) as i64;
        let across = point[0].wrapping_sub(from[0]) as i64 * to[1].wrapping_sub(from[1]) as i64;
        // Not on the inner side of this edge: the switch stays.
        if across.wrapping_sub(along) >= 0 {
            return;
        }
    }
    sun.on_screen.set(0);
}

/// The positions into the vertex buffer, mirrored for a mirrored view.
fn place(
    part: &crate::workers::PartScratch,
    quad: &[crate::session::Dword; 32],
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
    entries: &[Listed],
    mirrored: bool,
    window: &crate::scene::view_window::ViewWindow,
) {
    let buffer = part.vertices;
    let (x_offset, y_offset) = (
        window.view_offset_float[0].get(),
        window.view_offset_float[1].get(),
    );
    let depth_factor = window.one_over_farthest.get();
    let sse = scene.draw_switches.use_sse.get() != 0;
    let mirror_at = window.view_centre[0].get();
    for (n, &(_, entry)) in entries.iter().enumerate() {
        let vertex = entry.record.get();
        let out = out(buffer, n);
        let mut x = x87::add(
            crate::scene::point_records::vertex(part, quad, vertex, 0x0C),
            x_offset,
        );
        let depth = x87::mul(
            crate::scene::point_records::vertex(part, quad, vertex, 0x18),
            depth_factor,
        );
        out.y.set(x87::add(
            crate::scene::point_records::vertex(part, quad, vertex, 0x1C),
            y_offset,
        ));
        out.z.set(depth);
        out.rhw.set(if sse {
            x87::rcpss_times(
                crate::scene::point_records::vertex(part, quad, vertex, 0x18),
                race.course.numbers.k65536.get(),
            )
        } else {
            x87::reciprocal_times(depth, race.frame.panel_and_overlay.overlay.w_factor.get())
        });
        if mirrored && entries.len() > 2 {
            x = x87::mirrored(x, mirror_at, x_offset);
        }
        out.x.set(x);
    }
}

/// A set's texture coordinates, colour and specular into the buffer.
fn copy_set(buffer: &PolygonVertices, entries: &[Listed], set: Set) {
    let (uv, colour) = match set {
        Set::A => (0x08, 0x24),
        Set::B => (0x10, 0x2C),
    };
    for (n, &(_, entry)) in entries.iter().enumerate() {
        let out = out(buffer, n);
        out.u.set(x87::loaded(entry.word(uv)));
        out.v.set(x87::loaded(entry.word(uv + 4)));
        out.colour.set(entry.word(colour));
        out.specular.set(entry.word(colour + 4));
    }
}

/// `scene::choose_texture` with `scene::look_up_texture`: the texture filter's state, and the
/// polygon's texture from the number in `texture_page`.
fn choose_texture(
    poly: &crate::scene::polygon_work::PolygonWork,
    outside: &mut dyn Outside,
    scene: &crate::scene::owner::Scene,
) {
    let smooth = poly.smooth.get() == 1;
    outside.state(3, if smooth { 3 } else { 1 });
    let number: u32 = poly.texture_page.get();
    let loaded: u32 = scene.textures.places.totals[2].get();
    let texture = match number {
        u32::MAX => None,
        0x8000_0000.. => {
            let number = number & 0x7FFF_FFFF;
            (number < 0xFA0).then(|| scene.textures.places.place_slot[number as usize].get())
        }
        _ => (number < 0x400).then(|| scene.textures.kinds.kind_first_slots[number as usize].get()),
    };
    let texture = texture.filter(|&texture: &u32| texture < loaded);
    poly.polygon_texture.set(texture.unwrap_or(NO_TEXTURE));
}

/// Clears a switch by what the entries' bytes +0x21 hold.
fn clear_a_switch(
    tables: &crate::scene::polygon_store::PolygonTables,
    poly: &crate::scene::polygon_work::PolygonWork,
    face: &crate::race::face_work::FaceWork,
    entries: &[Listed],
) -> Result<(), BadInput> {
    if poly.polygon_sides[0].get() == 0 || poly.polygon_sides[1].get() == 0 {
        return Ok(());
    }
    let bytes = entries.iter().map(|&(_, entry)| entry.amount.get());
    let (all, any) = bytes.fold((0xFFu8, 0u8), |(all, any), b| (all & b, any | b));
    if all == 0xFF {
        let info = tables
            .texture_infos
            .get(face.texture_b.get() as usize)
            .ok_or(BadInput("a texture past the records"))?;
        // the record's byte +0x2E
        if info.words[0xB].get().to_le_bytes()[2] == 0 {
            poly.polygon_sides[0].set(0u32);
        }
    } else if any == 0 {
        poly.polygon_sides[1].set(0u32);
    }
    Ok(())
}

fn shade(
    poly: &crate::scene::polygon_work::PolygonWork,
    scene: &crate::scene::owner::Scene,
    entries: &[Listed],
    how: Shade,
) {
    let colour: u32 = poly.polygon_colour.get();
    for &(_, entry) in entries {
        let (fog, other) = (entry.fog.get() as u32, entry.amount.get() as u32);
        let put = |offset: usize, value: u32| entry.set_word(offset, value);
        match how {
            Shade::Plain => {
                put(0x24, colour);
                put(0x28, 0);
                continue;
            }
            Shade::WithAlpha => {
                put(0x24, scene.drawing.scene.polygon_alpha.get() << 24 | colour);
                put(0x28, 0);
                continue;
            }
            _ => {}
        }
        let lit = times(entry.light.get(), colour) | 0xFF00_0000;
        put(0x24, lit);
        put(0x28, fog << 24);
        match how {
            Shade::LitAndGrey => {
                put(0x2C, ((other * fog) >> 8) * 0x01_0101);
                put(0x30, 0xFF00_0000);
            }
            Shade::LitAndSecond(fogged) => {
                put(0x2C, lit & 0xFF_FFFF | other << 24);
                put(0x30, if fogged { fog << 24 } else { 0xFF00_0000 });
            }
            _ => {}
        }
    }
}

pub fn draw_polygon(
    poly: &crate::scene::polygon_work::PolygonWork,
    part: &crate::workers::PartScratch,
    race: &crate::race::owner::Race,
    face: &crate::race::face_work::FaceWork,
    outside: &mut dyn Outside,
    scene: &crate::scene::owner::Scene,
    screen: &crate::session::Screen,
) -> Result<(), BadInput> {
    let count: u32 = poly.polygon_vertex_count.get();
    if count == 0 || count > MOST {
        return Err(BadInput("a polygon of no vertices, or of too many"));
    }
    let buffer = part.vertices;
    if count as usize > buffer.vertices.len() {
        // The original writes the vertices on past the buffer.
        return Err(BadInput(
            "a polygon of more vertices than the vertex buffer holds",
        ));
    }
    let mut entries = entries(part.polygon, count)?;
    // Nothing has been written up to here.
    hit_test(part, &poly.quad_vertices, &poly.sun, &entries);
    let kept: [u32; 3] = [
        poly.polygon_sides[0].get(),
        poly.polygon_sides[1].get(),
        face.polygon_mode.get(),
    ];
    crate::session::write_bits(&poly.polygon_kept, kept);
    let mirrored = poly.view_mirrored.get() != 0;
    if mirrored && count > 2 {
        // The list's entries after the first, turned round.
        entries[1..].reverse();
        for (n, &(number, _)) in entries.iter().enumerate().skip(1) {
            crate::scene::polygon::list_cell(part.polygon, n).set(number);
        }
    }
    place(
        part,
        &poly.quad_vertices,
        race,
        scene,
        &entries,
        mirrored,
        &poly.window,
    );

    if face.faded.get() != 0 {
        let (mode, texture): (u32, u32) = (face.polygon_mode.get(), poly.texture_page.get());
        // The faded first pass: each vertex fades by its own fog byte.
        let (from, alpha): (u32, u32) = (poly.fade_from.get(), face.fade_alpha.get());
        let uv = crate::session::read_bits(&race.shared.vars.fade_uv);
        for (n, &(_, entry)) in entries.iter().enumerate() {
            let out = out(buffer, n);
            let by = entry.fog.get() as u32;
            out.colour
                .set(faded(from, 0x10, by) & 0xFF_FFFF | alpha << 24);
            out.specular.set(0xFF00_0000);
            out.u.set(f32::from_bits(uv[0]));
            out.v.set(f32::from_bits(uv[1]));
        }
        poly.texture_page.set(FADE_TEXTURE);
        face.polygon_mode.set(7u32);
        choose_texture(poly, outside, scene);
        for (state, value) in [(0x15, 4), (0x13, 5), (0x14, 6), (0x1B, 1), (0x1C, 0)] {
            outside.state(state, value);
        }
        submit_polygon(
            poly,
            buffer,
            outside,
            scene.objects.tables.d3d_drawing_on.get() != 0,
            screen,
        );
        poly.texture_page.set(texture);
        face.polygon_mode.set(mode);
    }

    // The mode, with its fallbacks.
    let mut mode: u32 = face.polygon_mode.get();
    if mode == 3 && scene.draw_switches.clears_switch.get() != 0 {
        clear_a_switch(race.polygon_tables(), poly, face, &entries)?;
    }
    if matches!(mode, 3..=5) {
        let fallback = if poly.polygon_sides[1].get() == 0 {
            Some(2)
        } else if poly.polygon_sides[0].get() == 0 {
            Some([8, 9, 0xA][mode as usize - 3])
        } else {
            None
        };
        if let Some(fallback) = fallback {
            mode = fallback;
            face.polygon_mode.set(mode);
        }
    }
    if let Some((how, passes)) = passes(mode) {
        shade(poly, scene, &entries, how);
        for pass in passes {
            let (texture, smooth) = match pass.set {
                Set::A => (&poly.polygon_textures[0], &poly.plain_first),
                Set::B => (&poly.polygon_textures[1], &poly.plain_second),
            };
            poly.texture_page.set(texture.get());
            poly.smooth.set(smooth.get());
            copy_set(buffer, &entries, pass.set);
            if let Some(as_mode) = pass.as_mode {
                face.polygon_mode.set(as_mode);
            }
            if pass.no_depth {
                outside.state(0x0E, 0);
            }
            choose_texture(poly, outside, scene);
            for &(state, value) in pass.states {
                outside.state(state, value);
            }
            submit_polygon(
                poly,
                buffer,
                outside,
                scene.objects.tables.d3d_drawing_on.get() != 0,
                screen,
            );
            if pass.as_mode.is_some() {
                face.polygon_mode.set(mode);
            }
            if pass.no_depth {
                outside.state(0x0E, 1);
            }
            for &(state, value) in pass.after {
                outside.state(state, value);
            }
        }
    }
    // The switches and the mode as they were.
    poly.polygon_sides[0].set(kept[0]);
    poly.polygon_sides[1].set(kept[1]);
    face.polygon_mode.set(kept[2]);
    Ok(())
}

/// The game's Direct3D layer and log, for drawing part `.0`.
pub struct Game<'a>(pub &'a crate::workers::PartScratch);

impl submit_polygon::Outside for Game<'_> {
    fn texture(&self) -> u32 {
        submit_polygon::Game(self.0).texture()
    }
    fn set_texture(&mut self, texture: u32) {
        submit_polygon::Game(self.0).set_texture(texture);
    }
    fn draw_fan(&mut self, vertices: &[crate::d3d::Vertex]) -> u32 {
        submit_polygon::Game(self.0).draw_fan(vertices)
    }
    fn bad_vertex(
        &mut self,
        poly: &crate::scene::polygon_work::PolygonWork,
        buffer: &PolygonVertices,
        screen: &crate::session::Screen,
    ) {
        submit_polygon::Game(self.0).bad_vertex(poly, buffer, screen);
    }
    fn not_drawn(&mut self, result: u32) {
        submit_polygon::Game(self.0).not_drawn(result);
    }
}

impl Outside for Game<'_> {
    fn state(&mut self, state: u32, value: u32) {
        crate::d3d::set_render_state::set_render_state(self.0, value, state);
    }
}

/// `draw_polygon` with the game's Direct3D layer (`scene::submit_polygon`'s), for Rust
/// callers.
pub fn draw_polygon_in_game(
    poly: &crate::scene::polygon_work::PolygonWork,
    part: &crate::workers::PartScratch,
    race: &Race,
    face: &crate::race::face_work::FaceWork,
    scene: &crate::scene::owner::Scene,
    screen: &crate::session::Screen,
) -> Result<(), BadInput> {
    draw_polygon(poly, part, race, face, &mut Game(part), scene, screen)
}
