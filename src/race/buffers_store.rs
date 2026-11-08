//! The image's zeroed tail, IDA 0x960000 .. 0xA8C180, as one static with the image's layout
//! (phase 13, 2026-10-30; it was the block `race::store::buffers`): its seven typed pieces
//! and the three holes between them, in order. One static and not ten: the original reaches
//! on past its pieces' ends by address (the clipper's vertices, the checkpoint tables read
//! by a number held to the count: `race::checkpoints_store::nth`), and such a reading stays
//! in the bytes the image has there.

use crate::d3d::vertices_store::PolygonVertices;
use crate::race::checkpoints_store::Checkpoints;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::scene::engine_tables_store::EngineTables;
use crate::scene::faces_store::Faces;
use crate::scene::polygon_store::PolygonWork;
use crate::screens::text_buffers_store::TextBuffers;
use crate::session::Bytes;
use core::mem::{offset_of, size_of};

/// IDA 0x960000 .. 0xA8C180.
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct Buffers {
    /// +0x0 (0x960000): `unk_960000`, the room `hud::draw_view_strip`'s original passes in ecx
    /// (the Rust version reads none of it).
    pub strip_room: Bytes<0x5A0>,
    /// +0x5A0 (0x9605A0): `d3d::vertices_store`.
    pub vertices: PolygonVertices,
    /// +0xDA0 (0x960DA0): `WideCharStr`, the film's name in wide characters
    /// (`gfx::open_film`'s original; the film player has its own).
    pub film_name_wide: Bytes<0x200>,
    /// +0xFA0 (0x960FA0): `race::viewer_buffers_store`.
    pub viewer: ViewerBuffers,
    /// +0x15510 (0x975510): `scene::polygon_store`.
    pub polygon: PolygonWork,
    /// +0x43730 (0x9A3730): `scene::faces_store`.
    pub faces: Faces,
    /// +0x83738 (0x9E3738): `race::checkpoints_store`.
    pub checkpoints: Checkpoints,
    /// +0xB2AE4 (0xA12AE4): `dword_A12AE4`, the mirror picture's room for the platform's
    /// upload (0x44E828's esi; `race::forget_view_textures`, 0x455E61).
    pub mirror_upload: Bytes<0x11000>,
    /// +0xC3AE4 (0xA23AE4): `scene::engine_tables_store`.
    pub engine_tables: EngineTables,
    /// +0x12BB54 (0xA8BB54): `screens::text_buffers_store`.
    pub text: TextBuffers,
}

/// IDA 0x960000, the first byte.
pub const START: usize = 0x960000;
/// IDA 0xA8C180, the end.
pub const END: usize = 0xA8C180;

const _: () = {
    use crate::{d3d, race, scene, screens};
    assert!(size_of::<Buffers>() == END - START);
    assert!(offset_of!(Buffers, vertices) == d3d::vertices_store::START - START);
    assert!(offset_of!(Buffers, viewer) == race::viewer_buffers_store::START - START);
    assert!(offset_of!(Buffers, polygon) == scene::polygon_store::START - START);
    assert!(offset_of!(Buffers, faces) == scene::faces_store::START - START);
    assert!(offset_of!(Buffers, checkpoints) == race::checkpoints_store::START - START);
    assert!(offset_of!(Buffers, mirror_upload) == race::checkpoints_store::END - START);
    assert!(offset_of!(Buffers, engine_tables) == scene::engine_tables_store::START - START);
    assert!(offset_of!(Buffers, text) == screens::text_buffers_store::START - START);
    assert!(screens::text_buffers_store::END == END);
};

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl Buffers {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes([0u8; END - START])
    }
}

/// Where IDA 0x960000 is now (the static).
#[cfg(test)]
pub fn home() -> usize {
    crate::pointers::game_address(core::ptr::from_ref(&race().shared.buffers)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buffers_have_their_home() {
        assert_eq!(home() % 16, 0);
        assert_eq!(crate::moved::place(START), home());
        assert_eq!(crate::moved::place(END - 1), home() + (END - START) - 1);
    }
}
