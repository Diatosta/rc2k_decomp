//! What `gfx::upload_texture_picture` (0x41A68B) and `scene::upload_car_icon` (0x41B2F2)
//! share: a system texture's record, the picture made ready for it, and how many
//! halvings apart the two are.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::owner::gfx;
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::{
    KEPT, alpha_from_kept, grade_texture, grading, key_texture_colours, least_colour,
};
use crate::session::Dword;

pub const RECORD_LEN: usize = 0x78;
pub const TEXTURES: u32 = 0x140;

/// A pixel kind's description, 0x40 bytes (`TextureFormat` in IDA): five from
/// `stru_5ABAD4` (555, 1555, 4444, 8888 and one more), pointed at by `off_5ABAC0`, filled
/// by `d3d::find_texture_formats`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TextureFormat {
    /// 1 when the device has the format.
    pub found: u32,
    pub bytes_per_pixel: u32,
    pub format: PixelFormat,
    /// The format's number (`gfx::describe_format` writes it after the masks).
    pub number: u32,
    /// DirectDraw's pixel format: size, flags, four-CC, bits, the masks red, green,
    /// blue, alpha.
    pub dd: [u32; 8],
}

const _: () = assert!(std::mem::size_of::<TextureFormat>() == 0x40);

/// A texture's record, 0x78 bytes (`TextureRecord` in IDA): the system textures from
/// `stru_5B521C`, the video textures from `stru_5ABC18`, 0x140 of each.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TextureRecord {
    /// 1 system, 2 video, 0 free.
    pub kind: u32,
    pub width: u32,
    pub height: u32,
    /// Its pixel kind's `TextureFormat`.
    pub format: u32,
    pub levels: u32,
    /// A DirectDraw surface a level.
    pub surfaces: [u32; 9],
    /// A Direct3D texture a level.
    pub textures: [u32; 9],
    /// 1: the alpha comes from a second picture.
    pub keeps_alpha: u32,
    pub locked: u32,
    /// A video texture: the system texture it holds.
    pub holds: u32,
    pub texture: u32,
    pub handle: u32,
    pub material: u32,
    pub material_handle: u32,
}

const _: () = assert!(std::mem::size_of::<TextureRecord>() == RECORD_LEN);

impl TextureRecord {
    /// The record's dwords, in the game's order.
    const fn words(&self) -> [u32; 30] {
        let mut w = [0u32; 30];
        w[0] = self.kind;
        w[1] = self.width;
        w[2] = self.height;
        w[3] = self.format;
        w[4] = self.levels;
        let mut n = 0;
        while n < 9 {
            w[5 + n] = self.surfaces[n];
            w[14 + n] = self.textures[n];
            n += 1;
        }
        w[23] = self.keeps_alpha;
        w[24] = self.locked;
        w[25] = self.holds;
        w[26] = self.texture;
        w[27] = self.handle;
        w[28] = self.material;
        w[29] = self.material_handle;
        w
    }

    fn from_words(w: [u32; 30]) -> Self {
        Self {
            kind: w[0],
            width: w[1],
            height: w[2],
            format: w[3],
            levels: w[4],
            surfaces: std::array::from_fn(|n| w[5 + n]),
            textures: std::array::from_fn(|n| w[14 + n]),
            keeps_alpha: w[23],
            locked: w[24],
            holds: w[25],
            texture: w[26],
            handle: w[27],
            material: w[28],
            material_handle: w[29],
        }
    }
}

/// A texture record kept as dwords (a pool's cell): read and written whole as
/// `Shared<TextureRecord>` was, through its cells.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct RecordCell([Dword; 30]);

impl RecordCell {
    pub const fn new(record: TextureRecord) -> Self {
        Self(crate::gfx::dwords(record.words()))
    }

    /// A copy of the record (its fields may be read half changed, as the original's).
    pub fn get(&self) -> TextureRecord {
        TextureRecord::from_words(crate::session::read_all(&self.0))
    }

    pub fn set(&self, record: TextureRecord) {
        crate::session::write_all(&self.0, record.words());
    }

    pub fn update(&self, change: impl FnOnce(&mut TextureRecord)) {
        let mut record = self.get();
        change(&mut record);
        self.set(record);
    }
}

impl TextureFormat {
    /// The description's dwords, in the game's order.
    const fn words(&self) -> [u32; 16] {
        let mut w = [0u32; 16];
        w[0] = self.found;
        w[1] = self.bytes_per_pixel;
        w[2] = self.format.layout;
        let mut n = 0;
        while n < 4 {
            w[3 + n] = self.format.masks[n];
            n += 1;
        }
        w[7] = self.number;
        let mut n = 0;
        while n < 8 {
            w[8 + n] = self.dd[n];
            n += 1;
        }
        w
    }

    fn from_words(w: [u32; 16]) -> Self {
        Self {
            found: w[0],
            bytes_per_pixel: w[1],
            format: PixelFormat {
                layout: w[2],
                masks: std::array::from_fn(|n| w[3 + n]),
            },
            number: w[7],
            dd: std::array::from_fn(|n| w[8 + n]),
        }
    }
}

/// A pixel kind's description kept as dwords (as `RecordCell`).
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct FormatCell([Dword; 16]);

impl FormatCell {
    pub const fn new(format: TextureFormat) -> Self {
        Self(crate::gfx::dwords(format.words()))
    }

    pub fn get(&self) -> TextureFormat {
        TextureFormat::from_words(crate::session::read_all(&self.0))
    }

    pub fn set(&self, format: TextureFormat) {
        crate::session::write_all(&self.0, format.words());
    }

    pub fn update(&self, change: impl FnOnce(&mut TextureFormat)) {
        let mut format = self.get();
        change(&mut format);
        self.set(format);
    }

    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(std::ptr::from_ref(self))
    }
}

/// IDA's address of the 3D card's tables.
pub const START: usize = 0x5ABA8C;
/// Their length.
pub const LEN: usize = 0x5BE81C - START;

/// A pool of texture records (the system's or the video's) and how many are made.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Pool {
    /// The textures made (`dword_5B5218` / `dword_5ABC14`), at most `TEXTURES`.
    pub count: Dword,
    /// The records (`stru_5B521C` / `stru_5ABC18`), 0x140 of 0x78 bytes; free ones hold -1
    /// in `holds`.
    pub records: [RecordCell; TEXTURES as usize],
}

/// The 3D card's tables and the textures' records.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Textures {
    /// `dword_5ABA8C`: 1 while the textures being made are of the alpha kind.
    pub alpha_kind: Dword,
    /// `dword_5ABA90`: the first video texture of the batch being made.
    pub first_video: Dword,
    /// `dword_5ABA94`: the first system texture of the batch being made.
    pub first_system: Dword,
    /// `dword_5ABA98`: five pairs of a size's number and the size (256, 128, 64, 32, 1024).
    pub sizes: [Dword; 10],
    /// `off_5ABAC0`: a pixel kind's description (by handle into `descriptions`,
    /// `format_ref`: the original kept its address; kind 0 none), filled by
    /// `d3d::find_texture_formats`.
    pub kinds: [Dword; 5],
    /// `stru_5ABAD4`: five descriptions of 0x40 bytes: 555, 1555, 4444, 8888 and the
    /// screen's (set by `platform::set_mode`).
    pub descriptions: [FormatCell; 5],
    /// The video textures (`dword_5ABC14`, `stru_5ABC18`).
    pub video: Pool,
    /// The system textures (`dword_5B5218`, `stru_5B521C`).
    pub system: Pool,
}

const FREE: TextureRecord = TextureRecord {
    kind: 0,
    width: 0,
    height: 0,
    format: 0,
    levels: 0,
    surfaces: [0; 9],
    textures: [0; 9],
    keeps_alpha: 0,
    locked: 0,
    holds: u32::MAX,
    texture: 0,
    handle: 0,
    material: 0,
    material_handle: 0,
};

const NO_FORMAT: TextureFormat = TextureFormat {
    found: 0,
    bytes_per_pixel: 0,
    format: PixelFormat {
        layout: 0,
        masks: [0; 4],
    },
    number: 0,
    dd: [0; 8],
};

const fn pool() -> Pool {
    Pool {
        count: Dword::new(0),
        records: [const { RecordCell::new(FREE) }; TEXTURES as usize],
    }
}

/// A texture format by handle (phase 13): `FORMAT_TAG | n` for `descriptions[n]`, 0 none.
pub const FORMAT_TAG: u32 = 0xCC00_0000;
/// IDA's address of `descriptions`.
const DESCRIPTIONS_IDA: u32 = 0x5ABAD4;
/// The handle for IDA's `ida` (a description's address or 0; anything else fails to
/// compile).
pub const fn format_ref(ida: u32) -> u32 {
    if ida == 0 {
        return 0;
    }
    assert!(ida >= DESCRIPTIONS_IDA && (ida - DESCRIPTIONS_IDA).is_multiple_of(0x40));
    assert!((ida - DESCRIPTIONS_IDA) / 0x40 < 5);
    FORMAT_TAG | ((ida - DESCRIPTIONS_IDA) / 0x40)
}

impl Textures {
    /// The 3D card's tables; the first values are the image's (the kinds by handle).
    pub const fn new() -> Self {
        Textures {
            alpha_kind: Dword::new(0),
            first_video: Dword::new(0),
            first_system: Dword::new(0),
            sizes: crate::gfx::dwords([0, 0x100, 1, 0x80, 2, 0x40, 3, 0x20, 4, 0x400]),
            kinds: crate::gfx::dwords([
                format_ref(0),
                format_ref(0x5ABAD4),
                format_ref(0x5ABB14),
                format_ref(0x5ABB54),
                format_ref(0x5ABBD4),
            ]),
            descriptions: [const { FormatCell::new(NO_FORMAT) }; 5],
            video: pool(),
            system: pool(),
        }
    }
}

impl Textures {
    /// A pool: 0 the system's, 1 the video's.
    pub fn pool(&self, pool: usize) -> &Pool {
        if pool == 0 { &self.system } else { &self.video }
    }

    /// Where kind `n`'s description is (0: none). A value that is no handle ends the
    /// program.
    // A number: the record's kind field is a game address other modules read.
    pub fn kind(&self, n: usize) -> u32 {
        let handle = self.kinds[n].get();
        if handle == 0 {
            return 0;
        }
        let k = (handle ^ FORMAT_TAG) as usize;
        if k >= self.descriptions.len() {
            crate::fatal::raise::end(
                "gfx::texture_upload::kind",
                BadInput("a texture kind's handle that names none"),
            );
        }
        self.descriptions[k].addr()
    }

    /// Kind `n`'s description's number in `descriptions` (None: none). A value that is no
    /// handle ends the program, as `kind`.
    pub fn kind_index(&self, n: usize) -> Option<usize> {
        let handle = self.kinds[n].get();
        if handle == 0 {
            return None;
        }
        let k = (handle ^ FORMAT_TAG) as usize;
        if k >= self.descriptions.len() {
            crate::fatal::raise::end(
                "gfx::texture_upload::kind_index",
                BadInput("a texture kind's handle that names none"),
            );
        }
        Some(k)
    }

    /// The kinds table's entry `n`, read where the original reads it (not checked against
    /// the table: a kind past it reads on into the descriptions' bytes).
    pub fn kind_entry(&self, n: usize) -> u32 {
        if n < self.kinds.len() {
            return self.kind(n);
        }
        // Bounded to the whole table (the kinds are followed by the descriptions).
        crate::pointers::GamePtr::to(self)
            .offset((core::mem::offset_of!(Textures, kinds) + 4 * n) as u32)
            .read::<u32>("a texture kind")
            .unwrap_or_else(|e| crate::fatal::raise::end("gfx::texture_upload::kind_entry", e))
    }
}

/// A system texture's record where it lies.
pub struct Record(&'static RecordCell);

impl Record {
    pub fn of(texture: u32) -> Result<Self, Refused> {
        if texture >= TEXTURES {
            return Err(BadInput("a texture past the records").into());
        }
        let record = Record(&gfx().card.texture_tables.system.records[texture as usize]);
        if record.get().format == 0 {
            return Err(BadInput("a texture's record without its format").into());
        }
        Ok(record)
    }

    pub fn get(&self) -> TextureRecord {
        self.0.get()
    }

    pub fn set_keeps_alpha(&self, keeps: u32) {
        self.0.update(|record| record.keeps_alpha = keeps);
    }

    fn texture_format(&self) -> Result<TextureFormat, BadInput> {
        let at = self.get().format;
        gfx()
            .card
            .texture_tables
            .descriptions
            .iter()
            .find(|d| d.addr() == at)
            .map(FormatCell::get)
            .ok_or(BadInput("a texture's format that names no description"))
    }

    pub fn format(&self) -> Result<PixelFormat, BadInput> {
        Ok(self.texture_format()?.format)
    }

    pub fn bytes_per_pixel(&self) -> Result<u32, BadInput> {
        Ok(self.texture_format()?.bytes_per_pixel)
    }

    pub fn keeps_alpha(&self) -> bool {
        self.get().keeps_alpha == 1
    }
}

pub fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

/// The least colour, then the alpha from the kept picture or the colours keyed, then
/// the grading.
pub fn ready_picture(record: &Record, pixels: u32, width: u32, height: u32) -> Result<(), Refused> {
    let gfx_grading = &gfx().grading;
    let count = width.wrapping_mul(height);
    if count == 0 {
        return Err(BadInput("a picture of no pixels").into());
    }
    if record.keeps_alpha() && count > 0x10000 {
        return Err(BadInput("more pixels than the kept picture's room").into());
    }
    {
        let key = least_colour(&record.format()?);
        if record.keeps_alpha() {
            alpha_from_kept(pixels, count)?;
        } else {
            key_texture_colours(pixels, count, key)?;
        }
        let (reading, scale, saturation) = (
            gfx_grading.reading.get(),
            gfx_grading.scale.get(),
            gfx_grading.saturation.get(),
        );
        if grading(reading, scale, saturation) {
            let mut table = [0u8; 0x100];
            grade_texture(pixels, count, key, scale, saturation, &mut table)?;
            gfx_grading.grades.write(&table);
        }
    }
    Ok(())
}

/// The number of halvings from 256 a size is, by the game's table.
fn halvings(size: u32) -> Option<u32> {
    let card = &gfx().card;
    (0..5).find_map(|n| {
        let (number, of) = (
            card.texture_tables.sizes[2 * n].get(),
            card.texture_tables.sizes[2 * n + 1].get(),
        );
        (of == size).then_some(number)
    })
}

/// How many halvings the picture is from the texture, the same across and down, or the
/// fatal error of the five `codes` (the picture's width, the texture's, the picture's
/// height, the texture's, the two not the same).
pub fn halvings_apart(
    record: &Record,
    width: u32,
    height: u32,
    codes: [u32; 5],
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::gfx::uploads::halvings_apart`.
    rc2k_core::gfx::uploads::halvings_apart(halvings, width, height, || {
        let r = record.get();
        (r.width, r.height)
    })
    .map_err(|which| fatal(codes[which as usize]))
}

/// With the record's +0x5C at 1: the second picture (`path`, 0 for none) read and
/// copied `KEPT` bytes on, or, if there is none or it cannot be read, +0x5C made 0.
pub fn keep_second_picture(
    record: &Record,
    path: Option<crate::pointers::GamePtr>,
) -> Result<(), Refused> {
    if !record.keeps_alpha() {
        return Ok(());
    }
    let kept = match path {
        None => None,
        Some(path) => match { read_texture_picture(path) }? {
            (_, _, _, true) => None,
            found => Some(found),
        },
    };
    match kept {
        Some((pixels, width, height, _)) => {
            // `rep movsd`; the texture size keeps the two apart (`KEPT` is 256 x 256 dwords).
            let len = width.wrapping_mul(height) as usize * 4;
            crate::gfx::picture::with_buffer(pixels, |b| {
                (KEPT.checked_add(len)? <= b.len()).then(|| b.copy_within(..len, KEPT))
            })?
            .ok_or(BadInput("a second texture picture past the picture buffer"))?;
        }
        None => record.set_keeps_alpha(0),
    }
    Ok(())
}

/// IDA's `off_5ABAC0`.
#[cfg(test)]
const KINDS_IDA: usize = 0x5ABAC0;

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games_but(
            &gfx().card.texture_tables,
            START,
            START + LEN,
            |image| {
                for k in 0..5 {
                    let o = KINDS_IDA - START + 4 * k;
                    let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                    image[o..o + 4].copy_from_slice(&format_ref(v).to_le_bytes());
                }
            },
        );
    }

    #[test]
    fn moved_knows_the_block() {
        let card = &gfx().card;
        assert_eq!(offset_of!(Textures, sizes), 0x5ABA98 - START);
        assert_eq!(offset_of!(Textures, kinds), 0x5ABAC0 - START);
        assert_eq!(offset_of!(Textures, descriptions), 0x5ABAD4 - START);
        assert_eq!(offset_of!(Textures, video), 0x5ABC14 - START);
        assert_eq!(offset_of!(Textures, system), 0x5B5218 - START);
        assert_eq!(offset_of!(Pool, records), 4);
        assert_eq!(
            crate::moved::place(START),
            card.texture_tables.alpha_kind.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5B521C),
            (crate::pointers::game_address(card.texture_tables.system.records.as_ptr())) as usize
        );
    }
}
