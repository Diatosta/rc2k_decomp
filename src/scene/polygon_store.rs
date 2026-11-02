//! The polygon work area, typed (phase 7): IDA 0x975510 .. 0x9A3730, in the image's zeroed
//! tail (`race::buffers_store`): the 64 work entries a polygon is built in, the entries kept
//! for the fan, the two number lists (the polygon's vertices by entry and the list before a
//! clip), the pool of point records the clip makes, the textures' records and the objects'
//! materials. Names are readings unless IDA's.

use crate::session::{Byte, Bytes, Dword};

/// A polygon work entry (IDA's `PolygonEntry`, 0x44 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
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
    /// +0x1C: the lamp light, added to the light before drawing (`race::course_faces`: a
    /// corner's; 0 for skidmarks and tinted quads). A reading.
    pub lamp_light: Dword,
    /// +0x20: the fog byte.
    pub fog: Byte,
    /// +0x21: the amount (alpha) byte.
    pub amount: Byte,
    /// +0x22: not named.
    pub pad_22: Bytes<2>,
    /// +0x24: work space of the clip (`scene::clip`).
    pub work: Bytes<32>,
}

impl PolygonEntry {
    /// An entry of zeros (the x86 tests' own).
    #[cfg(all(test, target_arch = "x86"))]
    pub const fn zeroed() -> Self {
        Self {
            point: Dword::new(0),
            record: Dword::new(0),
            u: Dword::new(0),
            v: Dword::new(0),
            u2: Dword::new(0),
            v2: Dword::new(0),
            light: Dword::new(0),
            lamp_light: Dword::new(0),
            fog: Byte::new(0),
            amount: Byte::new(0),
            pad_22: Bytes::new([0; 2]),
            work: Bytes::new([0; 32]),
        }
    }

    /// The dword at `offset` (a multiple of 4 below 0x44), for the callers that pick a
    /// field by an offset (the original's `[entry + reg]`).
    pub fn word(&self, offset: usize) -> u32 {
        match offset {
            0x0 => self.point.get(),
            0x4 => self.record.get(),
            0x8 => self.u.get(),
            0xC => self.v.get(),
            0x10 => self.u2.get(),
            0x14 => self.v2.get(),
            0x18 => self.light.get(),
            0x1C => self.lamp_light.get(),
            0x20 => u32::from_le_bytes([
                self.fog.get(),
                self.amount.get(),
                self.pad_22.get(0),
                self.pad_22.get(1),
            ]),
            0x24..0x44 if offset.is_multiple_of(4) => self.work.get_u32(offset - 0x24),
            _ => panic!("a polygon entry's dword at {offset:#x}"),
        }
    }

    /// The dword at `offset` set (as `word`).
    pub fn set_word(&self, offset: usize, value: u32) {
        match offset {
            0x0 => self.point.set(value),
            0x4 => self.record.set(value),
            0x8 => self.u.set(value),
            0xC => self.v.set(value),
            0x10 => self.u2.set(value),
            0x14 => self.v2.set(value),
            0x18 => self.light.set(value),
            0x1C => self.lamp_light.set(value),
            0x20 => {
                let [a, b, c, d] = value.to_le_bytes();
                self.fog.set(a);
                self.amount.set(b);
                self.pad_22.set(0, c);
                self.pad_22.set(1, d);
            }
            0x24..0x44 if offset.is_multiple_of(4) => self.work.set_u32(offset - 0x24, value),
            _ => panic!("a polygon entry's dword at {offset:#x}"),
        }
    }

    /// The byte at `offset` (below 0x44).
    pub fn byte(&self, offset: usize) -> u8 {
        self.word(offset & !3).to_le_bytes()[offset & 3]
    }

    /// The byte at `offset` set.
    pub fn set_byte(&self, offset: usize, value: u8) {
        let mut bytes = self.word(offset & !3).to_le_bytes();
        bytes[offset & 3] = value;
        self.set_word(offset & !3, u32::from_le_bytes(bytes));
    }

    /// The whole entry's bytes.
    pub fn bytes(&self) -> [u8; 0x44] {
        let mut bytes = [0; 0x44];
        for (k, dword) in bytes.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            *dword = self.word(k * 4).to_le_bytes();
        }
        bytes
    }

    /// Another entry's contents copied here.
    pub fn copy_from(&self, other: &PolygonEntry) {
        for o in (0..0x44).step_by(4) {
            self.set_word(o, other.word(o));
        }
    }
}

/// A texture's record (0x40 bytes: `scene::draw_polygon`, `race::course_faces`, `race::level_init`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TextureInfo {
    pub words: [Dword; 16],
}

/// An object kind's material record (0x80 bytes: `scene::set_object_material`, `physics::face_contact`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ObjectMaterial {
    pub words: [Dword; 32],
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
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

/// The piece, a field of `race::buffers_store::BUFFERS`.
pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = PolygonWork;
    fn deref(&self) -> &PolygonWork {
        &crate::race::buffers_store::BUFFERS.polygon
    }
}

pub static POLYGON_WORK: InBuffers = InBuffers;

/// IDA 0x975510, the first byte of the piece.
pub const START: usize = 0x975510;
/// IDA 0x9A3730, the end of the piece.
pub const END: usize = 0x9A3730;

const _: () = assert!(END - START == core::mem::size_of::<PolygonWork>());

/// Where IDA 0x975510 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::moved::place(START) as u32
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
            (piece_addr() + (END - START) as u32 - 1) as usize
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
