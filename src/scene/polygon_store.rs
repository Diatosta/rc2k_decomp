//! The polygon work area (in the image's zeroed tail, `race::buffers_store`): the 64 work entries a
//! polygon is built in, the entries kept for the fan, the two number lists (the polygon's vertices
//! by entry and the list before a clip), the pool of point records the clip makes, the textures'
//! records and the objects' materials.

use crate::errors::BadInput;
use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::{PlainByte, PlainDword};

/// A polygon work entry (0x44 bytes).
pub struct PolygonEntry {
    /// The scene point's number.
    pub point: PlainDword,
    /// The point record (`PointRecord`) of its projection.
    pub record: PlainDword,
    /// The texture u.
    pub u: PlainDword,
    /// The texture v.
    pub v: PlainDword,
    /// The second texture u.
    pub u2: PlainDword,
    /// The second texture v.
    pub v2: PlainDword,
    /// The light (colour).
    pub light: PlainDword,
    /// The lamp light, added to the light before drawing (`race::course_faces`: a
    /// corner's; 0 for skidmarks and tinted quads). A reading.
    pub lamp_light: PlainDword,
    /// The fog byte.
    pub fog: PlainByte,
    /// The amount (alpha) byte.
    pub amount: PlainByte,
    /// The two top bytes of the dword holding `fog` and `amount`; copied along with the
    /// entry, never given a meaning (padding).
    pub pad_22: PlainBuffer<2>,
    /// Work space of the clip (`scene::clip`).
    pub work: PlainBuffer<32>,
}

impl PolygonEntry {
    /// An entry of zeros.
    pub const fn new() -> Self {
        Self {
            point: PlainDword::new(0),
            record: PlainDword::new(0),
            u: PlainDword::new(0),
            v: PlainDword::new(0),
            u2: PlainDword::new(0),
            v2: PlainDword::new(0),
            light: PlainDword::new(0),
            lamp_light: PlainDword::new(0),
            fog: PlainByte::new(0),
            amount: PlainByte::new(0),
            pad_22: PlainBuffer::new([0; 2]),
            work: PlainBuffer::new([0; 32]),
        }
    }

    /// The dword at `offset` (a multiple of 4 below 0x44), for the callers that pick a field by
    /// an offset.
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

/// A texture's record (0x40 bytes: `scene::draw_polygon`, `race::course_faces`,
/// `race::level_init`).
#[derive(Clone)]
pub struct TextureInfo {
    pub words: [u32; 16],
}

/// An object kind's material record (0x80 bytes: `scene::set_object_material`,
/// `physics::face_contact`).
#[derive(Clone)]
pub struct ObjectMaterial {
    pub words: [u32; 32],
}

pub struct PolygonWork {
    /// The 64 work entries, then from entry 64 the 64 entries kept for the fan: the table
    /// `scene::draw_polygon` takes room for 0x80 (`scene::clip`, `scene::draw_object_polygon`,
    /// `race::car_shadow`, `race::skidmarks`).
    pub entries: [PolygonEntry; 0x80],
    /// The entries' numbers in drawing order (`scene::draw_polygon`, `scene::clip`).
    pub list: [PlainDword; 64],
    /// The list before a clip (`scene::clip`, `race::car_shadow`).
    pub old_list: [PlainDword; 64],
    /// 0x600 bytes between the old list and the record pool: a polygon of more than 128
    /// vertices lists on into them (`scene::polygon::list_cell`).
    pub reserved_2400: [PlainDword; 0x180],
    /// The point records a clip makes, 0x2020 bytes (`scene::clip`, `scene::draw_flat_quad`).
    pub record_pool: [PlainDword; 0x808],
}

/// The textures' records and the objects' materials, after the work: shared by every drawing
/// thread, unlike the work. 0x29000 bytes, written when a stage's or the front end's objects are
/// loaded (`race::level_init`, `scene::read_kind_records`), never while a drawing scope is open
/// (`session::DrawArc`, read through `PolygonTablesRef`).
#[derive(Clone)]
pub struct PolygonTables {
    /// 256 records of 0x40 bytes (`scene::draw_polygon`, `race::course_faces`,
    /// `physics::wheel_surfaces`, `race::level_init`); word 8 of a record is the kind of the
    /// object whose record it is.
    pub texture_infos: [TextureInfo; 256],
    /// By object kind, 0x80 bytes each, 1200 of them (`scene::set_object_material`,
    /// `scene::draw_described_object`, `physics::face_contact`, `drive::animate_lit_faces`).
    pub object_materials: [ObjectMaterial; 1200],
}

impl PolygonWork {
    /// The work area, zeroed as in the image: one for each drawing part
    /// (`scene::scratch_store::PartScratch::polygon`).
    pub const fn new() -> Self {
        Self {
            entries: [const { PolygonEntry::new() }; 0x80],
            list: [const { PlainDword::new(0) }; 64],
            old_list: [const { PlainDword::new(0) }; 64],
            reserved_2400: [const { PlainDword::new(0) }; 0x180],
            record_pool: [const { PlainDword::new(0) }; 0x808],
        }
    }
}

impl TextureInfo {
    /// A record of zeros.
    pub const fn new() -> Self {
        Self { words: [0; 16] }
    }
}

impl ObjectMaterial {
    /// A record of zeros.
    pub const fn new() -> Self {
        Self { words: [0; 32] }
    }
}

impl PolygonTables {
    /// The tables, zeroed as in the image.
    pub const fn new() -> Self {
        Self {
            texture_infos: [const { TextureInfo::new() }; 256],
            object_materials: [const { ObjectMaterial::new() }; 1200],
        }
    }
}

/// The tables as one reader finds them (the game's thread's own, or a drawing part's copy).
pub type PolygonTablesRef<'a> = crate::memory::drawn_table::DrawnTable<'a, PolygonTables>;

/// The dwords of the texture records.
const INFO_WORDS: usize = 256 * 16;

impl PolygonTables {
    /// Dword `n` of the texture records followed by the material records, if there is one.
    fn word(&self, n: usize) -> Option<u32> {
        match n.checked_sub(INFO_WORDS) {
            None => Some(self.texture_infos[n / 16].words[n % 16]),
            Some(m) => self
                .object_materials
                .get(m / 32)
                .map(|each| each.words[m % 32]),
        }
    }

    /// The bytes from byte `at` of the texture records followed by the material records, into
    /// `to` (little-endian within each dword). A byte past the materials is refused, and logged.
    pub fn read_bytes(&self, at: usize, to: &mut [u8], what: &str) -> Result<(), BadInput> {
        for (k, out) in to.iter_mut().enumerate() {
            let byte = at
                .checked_add(k)
                .ok_or(BadInput("a byte past the fields of its record"))?;
            let Some(word) = self.word(byte / 4) else {
                crate::memory::host_hooks::log(&format!(
                    "pointers: {what}: dword {}, past the {} dwords listed",
                    byte / 4,
                    INFO_WORDS + self.object_materials.len() * 32
                ));
                return Err(BadInput("a dword past the fields of its record"));
            };
            *out = word.to_le_bytes()[byte % 4];
        }
        Ok(())
    }

    /// As `read_bytes`, over the texture records alone.
    pub fn read_info_bytes(&self, at: usize, to: &mut [u8], what: &str) -> Result<(), BadInput> {
        for (k, out) in to.iter_mut().enumerate() {
            let byte = at
                .checked_add(k)
                .ok_or(BadInput("a byte past the fields of its record"))?;
            let Some(word) = (byte / 4 < INFO_WORDS)
                .then(|| self.texture_infos[byte / 64].words[(byte / 4) % 16])
            else {
                crate::memory::host_hooks::log(&format!(
                    "pointers: {what}: dword {}, past the {INFO_WORDS} dwords listed",
                    byte / 4
                ));
                return Err(BadInput("a dword past the fields of its record"));
            };
            *out = word.to_le_bytes()[byte % 4];
        }
        Ok(())
    }
}
