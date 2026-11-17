//! The tyre-change screen's drawing: `drive::tyre_change_screen`, `drive::tyre_screen`,
//! `drive::tyre_screen_parts`, `drive::tyre_turned_quad`, `drive::tyre_quad`. This file: the
//! places, the quads' shared steps and the tests' frame.

use crate::d3d::vertices_store::{PolygonVertices, Vertex};
use crate::errors::Refused;

use crate::math::x87_float::{int, ld, st, stf};
use crate::memory::session::PlainDword;
use crate::race::viewer_screen_store::ViewerPanelBlock;
pub use rc2k_core::hud::anchor::Spread;

/// `d3d::set_texture` on `part`.
pub fn set_texture(part: &crate::scene::workers::PartScratch, texture: u32) {
    crate::d3d::set_texture::set_texture(part, texture);
}

/// `d3d::set_render_state` on `part`.
pub fn set_render_state(part: &crate::scene::workers::PartScratch, value: u32, state: u32) {
    crate::d3d::set_render_state::set_render_state(part, value, state);
}

/// The quads' fixed words: z 0, rhw 1.0, the colour, the specular 0.
pub fn quad_words(buffer: &PolygonVertices, colour: u32) {
    for item in buffer.vertices.iter().take(4) {
        let v = &item;
        v.z.set(0.0);
        v.rhw.set(1.0);
        v.colour.set(colour);
        v.specular.set(0);
    }
}

/// The quads' shared tail: the texture places from the rectangle `uv` (x, y, w, h, whole
/// numbers) times the panel's texel scale, and the corners scaled to the screen (width and
/// height times the panel's two smaller scales, through the machine's scratch float).
pub fn quad_tail(
    buffer: &PolygonVertices,
    uv: [u32; 4],
    panel: &ViewerPanelBlock,
    system: &crate::system::owner::System,
    spread: Spread,
) {
    let session = &system.session;
    quad_tail_math_in(
        panel,
        uv,
        &buffer.vertices[..4],
        [
            session.screen.get().width.get(),
            session.screen.get().height.get(),
        ],
        &system.machine().scratch,
        spread,
    );
}

/// The tail's arithmetic: `rect` the texture rectangle (x, y, width, height, whole numbers),
/// `vertices` the four corners (left top, right top, left bottom, right bottom), `screen` the
/// screen size (whole numbers), `scratch` the float the screen scales pass through (stored as
/// a single).
fn quad_tail_math_in(
    panel: &ViewerPanelBlock,
    rect: [u32; 4],
    vertices: &[Vertex],
    screen: [u32; 2],
    scratch: &PlainDword,
    spread: Spread,
) {
    let [x, y, width, height] = rect.map(|n| int(n as i32));
    let texel = ld(panel.texel_scale.get());
    let (left, right) = (x * texel, (x + width) * texel);
    let (top, bottom) = (y * texel, (y + height) * texel);
    for (corner, vertex) in vertices[..4].iter().enumerate() {
        let at_right = corner % 2 == 1;
        let at_bottom = corner >= 2;
        vertex.u.set(stf(if at_right { right } else { left }));
        vertex.v.set(stf(if at_bottom { bottom } else { top }));
    }
    scratch.set(st(
        int(screen[0] as i32) * ld(panel.reciprocal_width_800.get())
    ));
    if spread != Spread::Fill && rc2k_core::hud::anchor::is_wide(screen[0], screen[1]) {
        // A screen wider than 4:3: the quad keeps its shape and lies by its spread.
        let (factor, shift) = spread.across(screen[0], screen[1]);
        for vertex in &vertices[..4] {
            vertex
                .x
                .set((f64::from(vertex.x.get()) * factor + shift) as f32);
        }
    } else {
        for vertex in &vertices[..4] {
            vertex.x.set(stf(ld(vertex.x.get()) * ld(scratch.get())));
        }
    }
    scratch.set(st(
        int(screen[1] as i32) * ld(panel.reciprocal_height_600.get())
    ));
    for vertex in &vertices[..4] {
        vertex.y.set(stf(ld(vertex.y.get()) * ld(scratch.get())));
    }
}

/// The tail's arithmetic on the game's panel (the tests').
#[cfg(test)]
fn quad_tail_math(rect: [u32; 4], vertices: &[Vertex], screen: [u32; 2], scratch: &PlainDword) {
    quad_tail_math_in(
        crate::test_support::own::race().viewer_panel(),
        rect,
        vertices,
        screen,
        scratch,
        Spread::Fill,
    );
}

/// The draw (`d3d::draw_vertex_strip`) of 4 vertices.
pub fn draw(part: &crate::scene::workers::PartScratch) -> Result<(), Refused> {
    crate::d3d::draw_vertex_strip::draw_vertex_strip(part, &part.vertices.vertices, 4)?;
    Ok(())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    #[test]
    fn quad_tail_matches_the_assembly() {
        let blocks = vec![Block {
            name: "quad_tail",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I), (8, In::I), (0xC, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[
                        (0, In::F),
                        (0x20, In::F),
                        (0x40, In::F),
                        (0x60, In::F),
                        (4, In::F),
                        (0x24, In::F),
                        (0x44, In::F),
                        (0x64, In::F),
                    ],
                    writes: &[
                        (0, Out::F),
                        (0x20, Out::F),
                        (0x40, Out::F),
                        (0x60, Out::F),
                        (4, Out::F),
                        (0x24, Out::F),
                        (0x44, Out::F),
                        (0x64, Out::F),
                        (0x18, Out::F),
                        (0x58, Out::F),
                        (0x38, Out::F),
                        (0x78, Out::F),
                        (0x1C, Out::F),
                        (0x3C, Out::F),
                        (0x5C, Out::F),
                        (0x7C, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C2C64, In::F), (0x1C2C68, In::F), (0x1C2C6C, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                // SAFETY: the buffers are the operands' (a rectangle's four dwords, four
                // vertices, the screen's two, the scratch dword), lent and aligned.
                let (rect, verts, screen, scratch) = unsafe {
                    (
                        *(crate::test_support::vectors::block_vectors::at(a[0]) as *const [u32; 4]),
                        &*(crate::test_support::vectors::block_vectors::at(a[1])
                            as *const [Vertex; 4]),
                        *(crate::test_support::vectors::block_vectors::at(a[2]) as *const [u32; 2]),
                        &*(crate::test_support::vectors::block_vectors::at(a[3])
                            as *const PlainDword),
                    )
                };
                quad_tail_math(rect, verts, screen, scratch);
            },
        }];
        sparse("drive.tyre_parts", &blocks, 1500, 205);
    }
}
