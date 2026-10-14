//! What `gfx::upload_texture_picture` (0x41A68B) and `scene::upload_plate` (0x41B2F2)
//! share: a system texture's record, the picture made ready for it, and how many
//! halvings apart the two are.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::gfx::pixel_format::PixelFormat;
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::{
    GRADES_RVA, KEPT, READING_RVA, SATURATION_RVA, SCALE_RVA, alpha_from_kept, grade_texture,
    grading, key_texture_colours, least_colour,
};

pub const SYSTEM_RECORDS_RVA: usize = rva(0x5B521C); // unk_5B521C
pub const RECORD_LEN: usize = 0x78;
pub const TEXTURES: u32 = 0x140;
const SIZES_RVA: usize = rva(0x5ABA98); // dword_5ABA98: five pairs of a number and a size

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

/// A system texture's record where it lies.
pub struct Record(*mut TextureRecord);

impl Record {
    pub fn of(texture: u32) -> Result<Self, Refused> {
        if texture >= TEXTURES {
            return Err(BadInput("a texture past the records").into());
        }
        let record = Record(
            (game_addr(SYSTEM_RECORDS_RVA) + texture as usize * RECORD_LEN) as *mut TextureRecord,
        );
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

    fn texture_format(&self) -> TextureFormat {
        unsafe { (self.get().format as *const TextureFormat).read_unaligned() }
    }

    pub fn format(&self) -> PixelFormat {
        self.texture_format().format
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        self.texture_format().bytes_per_pixel
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
pub unsafe fn ready_picture(
    record: &Record,
    pixels: u32,
    width: u32,
    height: u32,
) -> Result<(), Refused> {
    let count = width.wrapping_mul(height);
    if count == 0 {
        return Err(BadInput("a picture of no pixels").into());
    }
    if record.keeps_alpha() && count > 0x10000 {
        return Err(BadInput("more pixels than the kept picture's room").into());
    }
    unsafe {
        let key = least_colour(&record.format());
        if record.keeps_alpha() {
            alpha_from_kept(pixels as *mut u8, count);
        } else {
            key_texture_colours(pixels as *mut u32, count, key);
        }
        let (reading, scale, saturation) = (
            game_read(READING_RVA),
            game_read(SCALE_RVA),
            game_read(SATURATION_RVA),
        );
        if grading(reading, scale, saturation) {
            let mut table = [0u8; 0x100];
            grade_texture(pixels as *mut u8, count, key, scale, saturation, &mut table);
            game_write(GRADES_RVA, table);
        }
    }
    Ok(())
}

/// The number of halvings from 256 a size is, by the game's table.
fn halvings(size: u32) -> Option<u32> {
    (0..5).find_map(|n| unsafe {
        let [number, of]: [u32; 2] = game_read(SIZES_RVA + n * 8);
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
    let across = halvings(width).ok_or_else(|| fatal(codes[0]))?;
    let wide = halvings(record.get().width).ok_or_else(|| fatal(codes[1]))?;
    let across = (wide as i32 - across as i32).max(0) as u32;
    let down = halvings(height).ok_or_else(|| fatal(codes[2]))?;
    let high = halvings(record.get().height).ok_or_else(|| fatal(codes[3]))?;
    let down = (high as i32 - down as i32).max(0) as u32;
    if across != down {
        return Err(fatal(codes[4]));
    }
    Ok(across)
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
        match unsafe { read_texture_picture(path as *mut u8) }? {
            (_, _, _, true) => None,
            found => Some(found),
        }
    };
    match kept {
        Some((pixels, width, height, _)) => unsafe {
            let from = pixels as *const u32;
            let to = (pixels as usize + KEPT) as *mut u32;
            // `rep movsd`: one dword after another, forwards.
            for n in 0..width.wrapping_mul(height) as usize {
                to.add(n).write_unaligned(from.add(n).read_unaligned());
            }
        },
        None => record.set_keeps_alpha(0),
    }
    Ok(())
}
