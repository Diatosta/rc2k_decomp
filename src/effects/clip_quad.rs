//! `effects::clip_quad` (0x4B8A0B): a quad (four vertices of 0x20 bytes at ebp: +0 x, +4 y,
//! +8 .. +0x14 kept, +0x18 / +0x1C texture) clipped against one screen edge by the
//! settings' +8: 4 x at least `g_marksScreenMiddleX`, 3 x at most `g_marksScreenMiddleXLess`,
//! 2 y at least `g_marksScreenMiddleY`, 1 y at most `g_marksScreenMiddleYLess` (each test a
//! signed compare of the floats' bits, as the original's `sub; jl`). All in: CF clear,
//! nothing changed. All out: CF set. Else the run of inside vertices (from the first after
//! an outside one to the last) and the two crossings (the far edge's, then the near
//! edge's: the edge's coordinate the line, the other and the texture interpolated on the
//! x87, +8 .. +0x14 the outside vertex's) are built in `g_marksClipVertices` (`TLVertex`s) and copied over
//! the quad; `g_polygonVertexCount` and `dword_93CC94` their number; CF clear. Other
//! settings: CF clear, nothing changed.
//!
//! In:  the quad (the original's ebp: `g_polygonVertices` at every call; `g_marksSettings`)
//! Out: the result: true when all is outside (the original's CF); no caller reads a
//!      register it leaves
//!
//! Eight callers, each `jb` after it. Refuses null settings (a null quad, refused before,
//! cannot come: the quad is the vertices' own).

use crate::effects::clip::{CLIP, GLOW};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::scene::draw_store::DRAW;
/// Per mode 1 .. 4: the line's variable, its coordinate's offset, whether inside is at or
/// below it.
const EDGES: [(usize, u32, bool); 4] = [
    (3, 4, true),  // 1: g_marksScreenMiddleYLess
    (2, 4, false), // 2: g_marksScreenMiddleY
    (1, 0, true),  // 3: g_marksScreenMiddleXLess
    (0, 0, false), // 4: g_marksScreenMiddleX
];

fn get(a: GamePtr) -> Result<u32, BadInput> {
    a.read("a quad's vertex field")
}

/// The Rust version (see the header): whether all the quad is outside (the original's CF).
/// The quad is the polygon vertices' first four (`g_polygonVertices`, every caller's);
/// the clipped polygon (six at most) goes back there.
pub fn clip_quad() -> Result<bool, Refused> {
    {
        let settings: u32 = crate::effects::setup::SETTINGS.is_set.get();
        if settings == 0 {
            return Err(BadInput("null marks settings").into());
        }
        let q = GamePtr::to(&crate::d3d::vertices_store::VERTICES.vertices);
        let marks = crate::effects::setup::marks()?;
        let mode = marks.marks_layout.get();
        if !(1..=4).contains(&mode) {
            return Ok(false);
        }
        let (edge, coord, below) = EDGES[mode as usize - 1];
        let line: u32 = CLIP.edges[edge].bits();
        // The test of vertex k: inside.
        let t = |k: u32| -> Result<_, Refused> {
            Ok({
                let v = get(q.offset(0x20 * k + coord))? as i32;
                if below {
                    line as i32 >= v
                } else {
                    v >= line as i32
                }
            })
        };
        let (esi, edi);
        if !t(0)? {
            let mut first = None;
            for k in 1..=3 {
                if t(k)? {
                    first = Some(k);
                    break;
                }
            }
            let Some(first) = first else {
                return Ok(true);
            };
            edi = 0x20 * first;
            esi = if t(3)? {
                0x60
            } else if t(2)? {
                0x40
            } else {
                0x20
            };
        } else if !t(1)? {
            esi = 0;
            edi = if t(2)? {
                0x40
            } else if t(3)? {
                0x60
            } else {
                0
            };
        } else if !t(2)? {
            esi = 0x20;
            edi = if t(3)? { 0x60 } else { 0 };
        } else if !t(3)? {
            esi = 0x40;
            edi = 0;
        } else {
            return Ok(false);
        }
        let work = GamePtr::to(&CLIP.vertices);
        let mut ebx = 0u32;
        let mut k = edi;
        loop {
            q.offset(k)
                .copy_to(work.offset(ebx), 0x20, "a clipped vertex")?;
            ebx += 0x20;
            if k == esi {
                break;
            }
            k = (k + 0x20) & 0x7F;
        }
        let c = (esi + 0x20) & 0x7F;
        let d = (edi.wrapping_sub(0x20)) & 0x7F;
        let other = 4 - coord;
        for (inside, outside) in [(esi, c), (edi, d)] {
            let (s, o) = (q.offset(inside), q.offset(outside));
            let pairs = [
                [get(s.offset(other))?, get(o.offset(other))?],
                [get(s.offset(0x18))?, get(o.offset(0x18))?],
                [get(s.offset(0x1C))?, get(o.offset(0x1C))?],
            ];
            let v = x87::clip_lerp3(line, get(o.offset(coord))?, get(s.offset(coord))?, pairs);
            let w = work.offset(ebx);
            let put = |off: u32, x: u32| w.offset(off).write::<u32>(x, "a mark's field");
            put(coord, line)?;
            put(other, v[0])?;
            put(0x18, v[1])?;
            put(0x1C, v[2])?;
            for off in [8u32, 0xC, 0x10, 0x14] {
                put(off, get(o.offset(off))?)?;
            }
            ebx += 0x20;
        }
        work.copy_to(q, ebx as usize, "the clipped polygon")?;
        let count = ebx >> 5;
        DRAW.polygon_vertex_count.set_bits(count);
        GLOW.clip_count.set(count);
    }
    Ok(false)
}

/// `scene::draw_fan` (0x44EB32) of `g_polygonVertices` (plain since the scene's
/// conversion, 2026-10-23).
pub fn draw_fan() {
    crate::scene::draw_fan::draw_fan();
}
