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
//! The calls through `part` as `draw_flat_quad`'s (2 the clip, 3 / 5 the depth buffer, 4
//! the draw), in the game their Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose calls go to stand-ins, against the Rust version, quads
//! by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

const DRAWN_RVA: usize = rva(0x624134); // g_polygonDrawn
const VERTEX_RVA: usize = rva(0x623E40); // dword_623E40: four of 0x20
const RECORDS_RVA: usize = rva(0x975510); // g_polygonEntries: records of 0x44
const CORNERS_RVA: usize = rva(0x623E10); // left, right; dword_623E20 top, bottom
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const DEPTH_RVA: usize = rva(0x5C5E20); // dword_5C5E20
const COLOUR_IN_RVA: usize = rva(0x623DE8); // dword_623DE8, dword_623DEC
const COLOUR_RVA: usize = rva(0x6240CC); // g_polygonColour
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const POOL_RVA: usize = rva(0x977F10); // g_polygonRecordPool
const NEXT_RECORD_RVA: usize = rva(0x6240A4); // g_polygonNextRecord
const LIST_RVA: usize = rva(0x977710); // g_polygonList
const NEXT_ENTRY_RVA: usize = rva(0x624114); // g_polygonNextEntry
const MODE_RVA: usize = rva(0x63DAB4); // g_polygonMode
const TEXTURES_RVA: [usize; 2] = [rva(0x6240BC), rva(0x6240C0)];
const SIDES_RVA: [usize; 2] = [rva(0x6240E0), rva(0x6240E4)];
const RECORD: usize = 0x44;

fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_tinted_quad(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        set(DRAWN_RVA, 0);
        for n in 0..4 {
            let rec = RECORDS_RVA + RECORD * n;
            set(rec + 4, game_addr(VERTEX_RVA + 0x20 * n) as u32);
            set(rec + 8, 0);
            set(rec + 0xC, 0);
        }
        let float = |r: usize| {
            let v = x87::from_int(game_read(r));
            set(SCRATCH_RVA, v);
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
        let depth: u32 = game_read(DEPTH_RVA);
        for n in 0..4 {
            v(n, 0x18, depth);
        }
        let see: u8 = game_read(COLOUR_IN_RVA + 4);
        set(COLOUR_RVA, game_read(COLOUR_IN_RVA));
        for n in 0..4 {
            let rec = RECORDS_RVA + RECORD * n;
            set(rec, 0xFFFF_FFFF);
            game_write(rec + 0x21, see);
            set(rec + 0x18, 0xFF_FFFF);
            set(rec + 0x1C, 0);
            game_write(rec + 0x20, 0xFFu8);
        }
        set(COUNT_RVA, 4);
        g.esi = game_addr(POOL_RVA) as u32;
        set(NEXT_RECORD_RVA, g.esi);
        for n in 0..4u32 {
            set(LIST_RVA + 4 * n as usize, n);
        }
        set(LIST_RVA + 16, 0);
        set(NEXT_ENTRY_RVA, 4);
        [g.eax, g.ecx] = [0xFF, 4];
        crate::scene::draw_flat_quad::game_part(2, &mut g)?;
        if game_read::<u32>(COUNT_RVA) >= 3 {
            crate::scene::draw_flat_quad::game_part(3, &mut g)?;
            set(MODE_RVA, 3);
            set(TEXTURES_RVA[0], 0xFFFF_FFFF);
            set(TEXTURES_RVA[1], 0xFFFF_FFFF);
            set(SIDES_RVA[0], 0);
            set(SIDES_RVA[1], 1);
            crate::scene::draw_flat_quad::game_part(4, &mut g)?;
            set(MODE_RVA, 0);
            set(DRAWN_RVA, 1);
            crate::scene::draw_flat_quad::game_part(5, &mut g)?;
        }
        set(COLOUR_RVA, 0xFF_FFFF);
    }
    Ok(g)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_tinted_quad(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
