//! The HUD batch's variables outside the block (see `hud`), shared by its functions.

use crate::framework::rva;

pub const SHOWN_TEXTURE_RVA: usize = rva(0x6D14FE); // dword_6D14FE
pub const POLYGON_RVA: usize = rva(0x9605A0); // g_polygonVertices
pub const POLYGON_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
