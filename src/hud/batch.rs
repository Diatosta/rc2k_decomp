//! The HUD batch's variables (see `hud`), shared by its hooks.

use crate::framework::rva;

pub const VERTICES_RVA: usize = rva(0x75C324); // unk_75C324: 0x400 TLVertex
pub const INDICES_RVA: usize = rva(0x764324); // unk_764324: 0x600 u16
pub const VERTEX_COUNT_RVA: usize = rva(0x764F24); // dword_764F24
pub const INDEX_COUNT_RVA: usize = rva(0x764F28); // dword_764F28
pub const TEXTURE_RVA: usize = rva(0x75C034); // dword_75C034: the texture asked for
pub const BATCH_TEXTURE_RVA: usize = rva(0x75C038); // dword_75C038: the batch's (-1 none)
pub const SHOWN_TEXTURE_RVA: usize = rva(0x6D14FE); // dword_6D14FE
pub const POLYGON_RVA: usize = rva(0x9605A0); // g_polygonVertices
pub const POLYGON_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
