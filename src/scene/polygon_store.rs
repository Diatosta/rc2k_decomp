//! The polygon work area, typed (phase 7): IDA 0x975510 .. 0x9A3730, in the image's zeroed
//! tail (`race::store::buffers`): the 64 work entries a polygon is built in, the entries kept
//! for the fan, the two number lists (the polygon's vertices by entry and the list before a
//! clip), the pool of point records the clip makes, the textures' records and the objects'
//! materials. Names are readings unless IDA's.

use crate::session::{Byte, Bytes, Dword};

/// A polygon work entry (IDA's `PolygonEntry`, 0x44 bytes).
#[repr(C)]
pub struct PolygonEntry {
    /// +0x0: the scene point's number.
    pub point: Dword,
    /// +0x4: the point record (`PointRecord`) of its projection.
    pub record: Dword,
    /// +0x8: the texture u.
    pub u: Dword,
    /// +0xC: the texture v.
    pub v: Dword,
    /// +0x10: the second texture u.
    pub u2: Dword,
    /// +0x14: the second texture v.
    pub v2: Dword,
    /// +0x18: the light (colour).
    pub light: Dword,
    /// +0x1C: not named.
    pub unknown_1c: Dword,
    /// +0x20: the fog byte.
    pub fog: Byte,
    /// +0x21: the amount (alpha) byte.
    pub amount: Byte,
    /// +0x22: not named.
    pub pad_22: Bytes<2>,
    /// +0x24: work space of the clip (`scene::clip`).
    pub work: Bytes<32>,
}

/// A texture's record (0x40 bytes: `scene::draw_polygon`, `race::course_faces`, `race::level_init`).
#[repr(C)]
pub struct TextureInfo {
    pub words: [Dword; 16],
}

/// An object kind's material record (0x80 bytes: `scene::set_object_material`, `physics::face_contact`).
#[repr(C)]
pub struct ObjectMaterial {
    pub words: [Dword; 32],
}

#[repr(C)]
pub struct PolygonWork {
    /// +0x0 (0x975510): `g_polygonEntries`: `PolygonEntry[64]`, and from entry 64 (0x976610, `g_polygonKeptEntries`) the 64 entries kept for the fan: the table `scene::draw_polygon` takes room for 0x80 (`scene::clip`, `scene::draw_object_polygon`, `race::car_shadow`, `race::skidmarks`).
    pub entries: [PolygonEntry; 0x80],
    /// +0x2200 (0x977710): `g_polygonList`: the entries' numbers in drawing order (`scene::draw_polygon`, `scene::clip`).
    pub list: [Dword; 64],
    /// +0x2300 (0x977810): `g_polygonOldList`: the list before a clip (`scene::clip`, `race::car_shadow`).
    pub old_list: [Dword; 64],
    /// +0x2400 (0x977910): not named: nothing reads it.
    pub unknown_2400: [Dword; 0x180],
    /// +0x2a00 (0x977F10): `g_polygonRecordPool`: the point records a clip makes, 0x2020 bytes (`scene::clip`, `scene::draw_flat_quad`).
    pub record_pool: [Dword; 0x808],
    /// +0x4a20 (0x979F30): `g_textureInfos`: 256 records of 0x40 bytes (`scene::draw_polygon`, `race::course_faces`, `physics::wheel_surfaces`, `race::level_init`; `dword_979F50` is the kind of the object whose record it is, +0x20).
    pub texture_infos: [TextureInfo; 256],
    /// +0x8a20 (0x97DF30): `g_objectMaterials`: by object kind, 0x80 bytes each, 1200 of them (`scene::set_object_material`, `scene::draw_described_object`, `physics::face_contact`, `drive::animate_lit_faces`).
    pub object_materials: [ObjectMaterial; 1200],
}

pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = PolygonWork;
    fn deref(&self) -> &PolygonWork {
        // SAFETY: the block lives as long as the program and the piece is aligned to 4 (the
        // test checks); every field is an atomic or a cell.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const PolygonWork) }
    }
}

pub static POLYGON_WORK: InBuffers = InBuffers;

/// IDA 0x975510, the first byte of the piece.
pub const START: usize = 0x975510;
/// IDA 0x9A3730, the end of the piece.
pub const END: usize = 0x9A3730;

const _: () = assert!(END - START == core::mem::size_of::<PolygonWork>());

/// Where IDA 0x975510 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(core::mem::size_of::<PolygonEntry>(), 0x44);
        assert_eq!(offset_of!(PolygonEntry, fog), 0x20);
        assert_eq!(offset_of!(PolygonEntry, work), 0x24);
        assert_eq!(core::mem::size_of::<PolygonEntry>() * 64, 0x976610 - START);
        assert_eq!(offset_of!(PolygonWork, list), 0x977710 - START);
        assert_eq!(offset_of!(PolygonWork, old_list), 0x977810 - START);
        assert_eq!(offset_of!(PolygonWork, record_pool), 0x977F10 - START);
        assert_eq!(offset_of!(PolygonWork, texture_infos), 0x979F30 - START);
        assert_eq!(offset_of!(PolygonWork, object_materials), 0x97DF30 - START);
    }
}
