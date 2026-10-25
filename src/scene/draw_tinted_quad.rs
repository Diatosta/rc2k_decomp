//! `scene::draw_tinted_quad` (0x44B538): a flat quad on the screen in `dword_623DE8`'s
//! colour, see-through by `dword_623DEC`'s low byte, through the polygon pipeline (858
//! callers: the HUD's backdrops, the service area's screen, the menus): as
//! `scene::draw_flat_quad` but the texture places 0, the colour whole, each record's +0x18
//! 0xFFFFFF, +0x1C 0, +0x20 0xFF and +0x21 the see-through byte; no texture scaling; with
//! three vertices or more after the clip: `scene::depth_buffer_off`, mode 3,
//! `dword_6240BC` / `dword_6240C0` -1, `dword_6240E0` 0, `dword_6240E4` 1,
//! `scene::draw_polygon`, mode 0, `g_polygonDrawn` 1, `scene::depth_buffer_on`; the
//! colour back to white.
//!
//! In:  nothing (`dword_623DE8` .. `dword_623E24`, set by the caller)
//! Out: eax 0xFF, ecx 4, esi the record pool going into the calls; then as they leave them
//!
//! The calls as `draw_flat_quad`'s (the clip, the depth buffer, the draw), their Rust
//! versions.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;

const VERTEX_RVA: usize = rva(0x623E40); // dword_623E40: four of 0x20
const RECORDS_RVA: usize = rva(0x975510); // g_polygonEntries: records of 0x44
const CORNERS_RVA: usize = rva(0x623E10); // left, right; dword_623E20 top, bottom
const COLOUR_IN_RVA: usize = rva(0x623DE8); // dword_623DE8, dword_623DEC
const POOL_RVA: usize = rva(0x977F10); // g_polygonRecordPool
const LIST_RVA: usize = rva(0x977710); // g_polygonList
const MODE_RVA: usize = rva(0x63DAB4); // g_polygonMode
const RECORD: usize = 0x44;

fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// The Rust version (see the header). It reads no register, and what it leaves is read by
/// none of its 856 call sites (register liveness, 2026-10-23).
pub unsafe fn draw_tinted_quad() -> Result<(), Refused> {
    unsafe {
        DRAW.polygon_drawn.set_bits(0);
        for n in 0..4 {
            let rec = RECORDS_RVA + RECORD * n;
            set(rec + 4, game_addr(VERTEX_RVA + 0x20 * n) as u32);
            set(rec + 8, 0);
            set(rec + 0xC, 0);
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
        let see: u8 = game_read(COLOUR_IN_RVA + 4);
        DRAW.polygon_colour
            .set_bits(RACE_VARS.quad_colour[0].bits());
        for n in 0..4 {
            let rec = RECORDS_RVA + RECORD * n;
            set(rec, 0xFFFF_FFFF);
            game_write(rec + 0x21, see);
            set(rec + 0x18, 0xFF_FFFF);
            set(rec + 0x1C, 0);
            game_write(rec + 0x20, 0xFFu8);
        }
        DRAW.polygon_vertex_count.set_bits(4);
        DRAW.polygon_next_record
            .set_bits(game_addr(POOL_RVA) as u32);
        for n in 0..4u32 {
            set(LIST_RVA + 4 * n as usize, n);
        }
        set(LIST_RVA + 16, 0);
        DRAW.polygon_next_entry.set_bits(4);
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
        if DRAW.polygon_vertex_count.bits() >= 3 {
            crate::scene::depth_buffer_on::depth_buffer(false);
            set(MODE_RVA, 3);
            DRAW.polygon_textures[0].set_bits(0xFFFF_FFFF);
            DRAW.polygon_textures[1].set_bits(0xFFFF_FFFF);
            DRAW.polygon_sides[0].set_bits(0);
            DRAW.polygon_sides[1].set_bits(1);
            crate::scene::draw_polygon::draw_polygon_in_game()?;
            set(MODE_RVA, 0);
            DRAW.polygon_drawn.set_bits(1);
            crate::scene::depth_buffer_on::depth_buffer(true);
        }
        DRAW.polygon_colour.set_bits(0xFF_FFFF);
    }
    Ok(())
}
