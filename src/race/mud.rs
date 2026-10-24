//! `race::loadlevel_init`'s "render3d initmud" (IDA, 2026-10-16): 64 KB on
//! `stru_5C6F50` for the mud (`dword_623D50`), the car's mud file
//! (`files\b\mudfile.0xx`, xx the car's file number) read into it, the mud's state
//! cleared, and 256 colours sampled from a picture surface (`dword_5D0DEC`, locked and
//! unlocked through DirectDraw: the game's, `sub_41AA4B` / `sub_41AB02`).

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};

pub const MUD_LIST_RVA: usize = rva(0x5C6F50); // stru_5C6F50
pub const MUD_RVA: usize = rva(0x623D50); // dword_623D50
pub const MUD_LEN: u32 = 0x1_0000;
pub const LIST_TOTAL_RVA: usize = rva(0x5C6F90); // dword_5C6F90
pub const CAR_RVA: usize = rva(0x70F410); // dword_70F410: the car
const FILE_NUMBERS_RVA: usize = rva(0x5C5928); // dword_5C5928: 0x22 cars' file numbers
pub const MUD_PATH_RVA: usize = rva(0x6238F4); // "files\b\mudfile.0xx"
pub const MUD_PATH_DIGITS_RVA: usize = rva(0x623905);
pub const STATE_RVA: usize = rva(0x623D58); // dword_623D58 .. dword_623D9C
pub const STATE_DWORDS: usize = 18;
pub const STATE_MORE: [usize; 2] = [rva(0x62390C), rva(0x623918)];
pub const PICTURE_RVA: usize = rva(0x5D0DEC); // dword_5D0DEC: the surface sampled
pub const COLOURS_RVA: usize = rva(0x623930); // 256 dwords
const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const HEIGHT_RVA: usize = rva(0x4F49A4); // g_surface.height
const FORMAT_RVA: usize = rva(0x4F49B8); // g_surface.format
const PITCH_RVA: usize = rva(0x4F49BC); // g_surface.pitch
const PIXELS_RVA: usize = rva(0x4F49C0); // g_surface.pixels
const BITS_RVA: usize = rva(0x4F49A8); // g_surface.bits_per_pixel

/// `race::car_file_number` (0x41F4FD): eax held to 0 .. 0x21 (signed), its entry of
/// `dword_5C5928`.
pub fn car_file_number(car: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    unsafe { game_read(FILE_NUMBERS_RVA + 4 * n) }
}

/// `gfx::read_pixel` (0x406584): the pixel at (ecx, edx) of `g_surface`, taken apart by
/// `gfx::pixel_to_components`. A 16-bit pixel is read with `mov ax`: the upper half of eax
/// is the row's address's, and that is what the conversion gets.
pub unsafe fn read_pixel(x: u32, y: u32) -> u32 {
    unsafe {
        let (_, pixel) = raw_pixel(x, y);
        let format = game_read::<u32>(FORMAT_RVA) as *const crate::gfx::pixel_format::PixelFormat;
        crate::gfx::pixel_to_components::pixel_to_components(&format.read_unaligned(), pixel)
    }
}

/// The row's address and the value `read_pixel` hands to the conversion (one place for
/// both: a second copy of this compiled without the row's upper half, 2026-10-16).
pub unsafe fn raw_pixel(x: u32, y: u32) -> (u32, u32) {
    unsafe {
        let row = game_read::<u32>(PIXELS_RVA).wrapping_add(y.wrapping_mul(game_read(PITCH_RVA)));
        let pixel = if game_read::<u32>(BITS_RVA) == 0x10 {
            // As the original: eax the row's address, then `mov ax` of the pixel. Written in
            // assembly: the optimiser dropped the row's half from the same expression in
            // Rust in one of two places (2026-10-16).
            let mut eax = row;
            core::arch::asm!(
                "mov ax, word ptr [{at}]",
                at = in(reg) row.wrapping_add(x.wrapping_mul(2)),
                inout("eax") eax,
                options(nostack, readonly, preserves_flags),
            );
            eax
        } else {
            (row.wrapping_add(x.wrapping_mul(4)) as *const u32).read_unaligned()
        };
        (row, pixel)
    }
}

/// `race::mud_pixel` (0x44A58A): `read_pixel` inside the surface (unsigned), else 0.
pub unsafe fn mud_pixel(x: u32, y: u32) -> u32 {
    unsafe {
        // Converted: `rc2k_core::race::mud::inside`.
        if !rc2k_core::race::mud::inside(x, y, game_read(WIDTH_RVA), game_read(HEIGHT_RVA)) {
            return 0;
        }
        read_pixel(x, y)
    }
}

/// `race::load_mud` (0x449D95): the car's file number's two digits (`div 10`, each +
/// '0', as bytes) into the path, the file (0x10000 bytes at most) into the mud.
pub unsafe fn load_mud() -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::mud::file_digits`.
        let (tens, units) = rc2k_core::race::mud::file_digits(car_file_number(game_read(CAR_RVA)));
        let digits = game_addr(MUD_PATH_DIGITS_RVA) as *mut u8;
        *digits = tens as u8;
        *digits.add(1) = units as u8;
        let mud: u32 = game_read(MUD_RVA);
        crate::files::read_whole_file::read_whole_file(
            game_addr(MUD_PATH_RVA) as *const u8,
            mud as *mut u8,
            MUD_LEN,
            mud,
        )?;
    }
    Ok(())
}

/// `race::clear_mud_state` (0x449E01): the 18 dwords at 0x623D58 and the two more 0.
pub unsafe fn clear_mud_state() {
    unsafe {
        game_write(STATE_RVA, [0u32; STATE_DWORDS]);
        for at in STATE_MORE {
            game_write(at, 0u32);
        }
    }
}

/// `race::sample_mud_colours` (0x44A0D6): the surface locked (`d3d::lock_texture`), 256
/// colours, n's at (0x89 + (n & 3) + (n >> 5) * 4, 0x68 + ((n & 0x1C) >> 2)) by
/// `mud_pixel`, unlocked (`d3d::unlock_texture`).
pub unsafe fn sample_mud_colours() -> Result<(), Refused> {
    unsafe {
        crate::d3d::lock_texture::lock_texture(game_read(PICTURE_RVA))?;
        let colours = game_addr(COLOURS_RVA) as *mut u32;
        // Converted: `rc2k_core::race::mud::sample_place`.
        for n in 0..rc2k_core::race::mud::SAMPLES {
            let (x, y) = rc2k_core::race::mud::sample_place(n);
            colours.add(n as usize).write_unaligned(mud_pixel(x, y));
        }
        crate::d3d::unlock_texture::unlock_texture(game_read(PICTURE_RVA))?;
    }
    Ok(())
}

/// `race::init_mud` (0x449DD6): the list's total + 0x10000, the mud taken
/// (`memory::allocate`) into `dword_623D50`, then `load_mud`, `clear_mud_state`,
/// `sample_mud_colours`.
pub unsafe fn init_mud() -> Result<(), Refused> {
    unsafe {
        game_write(
            LIST_TOTAL_RVA,
            game_read::<u32>(LIST_TOTAL_RVA).wrapping_add(MUD_LEN),
        );
        let mud = crate::memory::allocate::allocate(game_addr(MUD_LIST_RVA) as *mut _, MUD_LEN)?;
        game_write(MUD_RVA, mud as u32);
        load_mud()?;
        clear_mud_state();
        sample_mud_colours()
    }
}
