//! `scene::draw_flat_quad` (0x44AF85): a flat quad on the screen through the polygon
//! pipeline: `g_polygonDrawn` 0; the four polygon records' vertex pointers
//! (`dword_623E40` .. 0x20 apart), texture places (`dword_623DF0` .. u, `dword_623E00` ..
//! v), the corners' x (`dword_623E10` left, `dword_623E14` right) and y (`dword_623E20`
//! top, `dword_623E24` bottom) as floats (through `dword_62400C`), depth `dword_5C5E20`;
//! the entries -1, colour `dword_623DE8`'s low 24 bits, the records' +0x18 0; four
//! vertices, the record pool, the list 0 .. 3; then `scene::scale_texture_coordinates`,
//! `scene::clip_polygon_to_screen` and, with three vertices or more left, mode 6
//! (`g_polygonMode`), `dword_6240E0` 1, `dword_6240E4` 0, `scene::depth_buffer_off`,
//! `scene::draw_polygon`, `g_polygonDrawn` 1, `scene::depth_buffer_on`; the colour back to
//! white.
//!
//! In:  nothing (`dword_623DE8` .. `dword_623E24`, set by the caller)
//! Out: eax 0, ecx 4, esi the record pool going into the calls; then as they leave them
//!
//! Callers: `sub_45B86E` (three), and code at 0x4436B2, 0x4437C1, 0x45BCD8, 0x45BDD0. The
//! calls (the scale, the clip, the depth buffer off, the draw, the depth buffer on) are
//! their Rust versions; a refusal of theirs comes after the writes.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;

const VERTEX_RVA: usize = rva(0x623E40); // dword_623E40: four of 0x20
const RECORDS_RVA: usize = rva(0x975510); // g_polygonEntries: records of 0x44
const UV_RVA: usize = rva(0x623DF0); // dword_623DF0 .. dword_623E0C
const CORNERS_RVA: usize = rva(0x623E10); // left, right; dword_623E20 top, bottom
const POOL_RVA: usize = rva(0x977F10); // g_polygonRecordPool
const LIST_RVA: usize = rva(0x977710); // g_polygonList
const MODE_RVA: usize = rva(0x63DAB4); // g_polygonMode
const RECORD: usize = 0x44;

fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// The Rust version (see the header). It reads no register, and what it leaves is read by
/// none of its callers (register liveness, 2026-10-23).
pub unsafe fn draw_flat_quad() -> Result<(), Refused> {
    unsafe {
        DRAW.polygon_drawn.set_bits(0);
        for n in 0..4 {
            let rec = RECORDS_RVA + RECORD * n;
            set(rec + 4, game_addr(VERTEX_RVA + 0x20 * n) as u32);
            set(rec + 8, game_read(UV_RVA + 4 * n));
            set(rec + 0xC, game_read(UV_RVA + 0x10 + 4 * n));
        }
        let float = |r: usize| {
            let v = x87::from_int(game_read(r));
            DRAW.scratch.set_bits(v);
            v
        };
        let [left, right, top, bottom] = [
            CORNERS_RVA,
            CORNERS_RVA + 4,
            CORNERS_RVA + 0x10,
            CORNERS_RVA + 0x14,
        ]
        .map(float);
        let v = |n: usize, o: usize, x: u32| set(VERTEX_RVA + 0x20 * n + o, x);
        for (n, x, y) in [
            (0, left, top),
            (1, right, top),
            (2, right, bottom),
            (3, left, bottom),
        ] {
            v(n, 0xC, x);
            v(n, 0x1C, y);
        }
        let depth: u32 = NUMBERS.k4096_b.bits();
        for n in 0..4 {
            v(n, 0x18, depth);
        }
        for n in 0..4 {
            set(RECORDS_RVA + RECORD * n, 0xFFFF_FFFF);
        }
        DRAW.polygon_colour
            .set_bits(RACE_VARS.quad_colour[0].bits() & 0xFF_FFFF);
        for n in 0..4 {
            set(RECORDS_RVA + RECORD * n + 0x18, 0);
        }
        DRAW.polygon_vertex_count.set_bits(4);
        DRAW.polygon_next_record
            .set_bits(game_addr(POOL_RVA) as u32);
        for n in 0..4u32 {
            set(LIST_RVA + 4 * n as usize, n);
        }
        set(LIST_RVA + 16, 0);
        DRAW.polygon_next_entry.set_bits(4);
        crate::scene::scale_texture_coordinates::scale_texture_coordinates()?;
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
        if DRAW.polygon_vertex_count.bits() >= 3 {
            set(MODE_RVA, 6);
            DRAW.polygon_sides[0].set_bits(1);
            DRAW.polygon_sides[1].set_bits(0);
            crate::scene::depth_buffer_on::depth_buffer(false);
            crate::scene::draw_polygon::draw_polygon_in_game()?;
            DRAW.polygon_drawn.set_bits(1);
            crate::scene::depth_buffer_on::depth_buffer(true);
        }
        DRAW.polygon_colour.set_bits(0xFF_FFFF);
    }
    Ok(())
}
