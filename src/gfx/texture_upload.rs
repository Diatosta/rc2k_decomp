//! What `gfx::upload_texture_picture` (0x41A68B) and `scene::upload_car_icon` (0x41B2F2)
//! share: a system texture's record, the picture made ready for it, and how many
//! halvings apart the two are.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::GRADING;
use crate::gfx::texture_pixels::{
    KEPT, alpha_from_kept, grade_texture, grading, key_texture_colours, least_colour,
};
use crate::pointers::GamePtr;
use crate::session::{Dword, Shared};

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

/// IDA's address of the 3D card's tables.
pub const START: usize = 0x5ABA8C;
/// Their length.
pub const LEN: usize = 0x5BE81C - START;

/// A pool of texture records (the system's or the video's) and how many are made.
#[repr(C)]
pub struct Pool {
    /// The textures made (`dword_5B5218` / `dword_5ABC14`), at most `TEXTURES`.
    pub count: Dword,
    /// The records (`stru_5B521C` / `stru_5ABC18`), 0x140 of 0x78 bytes; free ones hold -1
    /// in `holds`.
    pub records: [Shared<TextureRecord>; TEXTURES as usize],
}

/// The 3D card's tables and the textures' records.
#[repr(C)]
pub struct Textures {
    /// `dword_5ABA8C`: 1 while the textures being made are of the alpha kind.
    pub alpha_kind: Dword,
    /// `dword_5ABA90`: the first video texture of the batch being made.
    pub first_video: Dword,
    /// `dword_5ABA94`: the first system texture of the batch being made.
    pub first_system: Dword,
    /// `dword_5ABA98`: five pairs of a size's number and the size (256, 128, 64, 32, 1024).
    pub sizes: [Dword; 10],
    /// `off_5ABAC0`: a pixel kind's description (a pointer into `descriptions`; kind 0
    /// none), filled by `d3d::find_texture_formats`.
    pub kinds: [Dword; 5],
    /// `stru_5ABAD4`: five descriptions of 0x40 bytes: 555, 1555, 4444, 8888 and the
    /// screen's (set by `platform::set_mode`).
    pub descriptions: [Shared<TextureFormat>; 5],
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

const NO_FORMAT: TextureFormat = unsafe { core::mem::zeroed() };

const fn pool() -> Pool {
    Pool {
        count: Dword::new(0),
        records: [const { Shared::new(FREE) }; TEXTURES as usize],
    }
}

/// The 3D card's tables; the first values are the image's (the kinds point at the
/// image's descriptions until `moved::point_into_homes`).
pub static TEXTURE_TABLES: Textures = Textures {
    alpha_kind: Dword::new(0),
    first_video: Dword::new(0),
    first_system: Dword::new(0),
    sizes: crate::gfx::dwords([0, 0x100, 1, 0x80, 2, 0x40, 3, 0x20, 4, 0x400]),
    kinds: crate::gfx::dwords([0, 0x5abad4, 0x5abb14, 0x5abb54, 0x5abbd4]),
    descriptions: [const { Shared::new(NO_FORMAT) }; 5],
    video: pool(),
    system: pool(),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x5ABA8C).
pub fn home() -> usize {
    crate::pointers::host_number(&TEXTURE_TABLES as *const Textures)
}

impl Textures {
    /// A pool: 0 the system's, 1 the video's.
    pub fn pool(&self, pool: usize) -> &Pool {
        if pool == 0 { &self.system } else { &self.video }
    }

    /// Where the sizes table is (five entries of two dwords, for `system::find_entry`).
    pub fn sizes_addr(&self) -> usize {
        crate::pointers::host_number(self as *const Self) + core::mem::offset_of!(Self, sizes)
    }

    /// The kinds table's entry `n`, read where the original reads it (not checked against
    /// the table: a kind past it reads on into the descriptions).
    pub fn kind_entry(&self, n: usize) -> u32 {
        unsafe { (self.kinds.as_ptr().cast::<u32>().add(n)).read_volatile() }
    }
}

/// A system texture's record where it lies.
pub struct Record(*mut TextureRecord);

impl Record {
    pub fn of(texture: u32) -> Result<Self, Refused> {
        if texture >= TEXTURES {
            return Err(BadInput("a texture past the records").into());
        }
        let record = Record(TEXTURE_TABLES.system.records[texture as usize].as_ptr());
        if record.get().format == 0 {
            return Err(BadInput("a texture's record without its format").into());
        }
        Ok(record)
    }

    pub fn get(&self) -> TextureRecord {
        unsafe { self.0.read_unaligned() }
    }

    pub fn set_keeps_alpha(&self, keeps: u32) {
        let mut record = self.get();
        record.keeps_alpha = keeps;
        unsafe { self.0.write_unaligned(record) }
    }

    fn texture_format(&self) -> Result<TextureFormat, BadInput> {
        GamePtr::new(self.get().format).read("a texture's format")
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
            GRADING.reading.get(),
            GRADING.scale.get(),
            GRADING.saturation.get(),
        );
        if grading(reading, scale, saturation) {
            let mut table = [0u8; 0x100];
            grade_texture(pixels, count, key, scale, saturation, &mut table)?;
            GRADING.grades.write(&table);
        }
    }
    Ok(())
}

/// The number of halvings from 256 a size is, by the game's table.
fn halvings(size: u32) -> Option<u32> {
    (0..5).find_map(|n| {
        let (number, of) = (
            TEXTURE_TABLES.sizes[2 * n].get(),
            TEXTURE_TABLES.sizes[2 * n + 1].get(),
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
pub unsafe fn keep_second_picture(record: &Record, path: u32) -> Result<(), Refused> {
    if !record.keeps_alpha() {
        return Ok(());
    }
    let kept = if path == 0 {
        None
    } else {
        match unsafe { read_texture_picture(crate::pointers::host(path)) }? {
            (_, _, _, true) => None,
            found => Some(found),
        }
    };
    match kept {
        Some((pixels, width, height, _)) => {
            // `rep movsd`: one dword after another, forwards.
            GamePtr::new(pixels).copy_to(
                GamePtr::new(pixels).offset(KEPT as u32),
                width.wrapping_mul(height) as usize * 4,
                "a second texture picture",
            )?;
        }
        None => record.set_keeps_alpha(0),
    }
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&TEXTURE_TABLES, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Textures, sizes), 0x5ABA98 - START);
        assert_eq!(offset_of!(Textures, kinds), 0x5ABAC0 - START);
        assert_eq!(offset_of!(Textures, descriptions), 0x5ABAD4 - START);
        assert_eq!(offset_of!(Textures, video), 0x5ABC14 - START);
        assert_eq!(offset_of!(Textures, system), 0x5B5218 - START);
        assert_eq!(offset_of!(Pool, records), 4);
        assert_eq!(crate::moved::place(START), TEXTURE_TABLES.alpha_kind.addr());
        assert_eq!(
            crate::moved::place(0x5B521C),
            crate::pointers::host_number(TEXTURE_TABLES.system.records.as_ptr())
        );
    }
}
