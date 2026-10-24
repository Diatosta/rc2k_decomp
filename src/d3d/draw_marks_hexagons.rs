//! `d3d::draw_marks_hexagons` (0x41ADB9): draws the marks' hexagons, an indexed list of
//! triangles.
//!
//! In:  nothing: the vertices at `unk_625A44`, their number in `dword_625A40`, the indices
//!      (16 bits) at `g_marksHexagonFans`, their number in `dword_62423C`
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's (`g_d3dDevice`, an `IDirect3DDevice2`) DrawIndexedPrimitive (+0x78) with
//! a triangle list (4) of transformed and lit vertices (3) and no flags. Callers:
//! `marks::hexagons_top`, `marks::hexagons_frame`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_indexed_list`) and the result is 0 (before 2026-10-19
//! the standalone skipped it). No self-test.

use crate::gfx::block::{game_addr, game_read};
use crate::gfx::hexagon_batch::Slot as HexSlot;

const VERTICES_RVA: HexSlot = HexSlot::at(0x625A44); // unk_625A44
const VERTEX_COUNT_RVA: HexSlot = HexSlot::at(0x625A40); // dword_625A40
const INDICES_RVA: HexSlot = HexSlot::at(0x624240); // g_marksHexagonFans
const INDEX_COUNT_RVA: HexSlot = HexSlot::at(0x62423C); // dword_62423C

/// The hexagons drawn: Direct3D's result, always 0 here.
pub fn draw_marks_hexagons() -> u32 {
    unsafe {
        crate::platform_3d::draw_indexed_list(
            game_addr(VERTICES_RVA) as u32,
            game_read(VERTEX_COUNT_RVA),
            game_addr(INDICES_RVA) as u32,
            game_read(INDEX_COUNT_RVA),
        )
    };
    0
}
