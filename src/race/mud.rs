//! `race::loadlevel_init`'s "render3d initmud" (IDA, 2026-10-16): 64 KB on
//! `stru_5C6F50` for the mud (`dword_623D50`), the car's mud file
//! (`files\b\mudfile.0xx`, xx the car's file number) read into it, the mud's state
//! cleared, and 256 colours sampled from a picture surface (`dword_5D0DEC`, locked and
//! unlocked through DirectDraw: the game's, `sub_41AA4B` / `sub_41AB02`).

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::kind_store::KIND_TABLES;
use crate::race::vars_store::RACE_VARS;
use crate::scene::kind_texture_store::KIND_TEXTURES;

pub const MUD_LEN: u32 = 0x1_0000;

/// `race::car_file_number` (0x41F4FD): eax held to 0 .. 0x21 (signed), its entry of
/// `dword_5C5928`.
pub fn car_file_number(car: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    KIND_TABLES.car_file_numbers[n].get()
}

/// `gfx::read_pixel` (0x406584): the pixel at (ecx, edx) of `g_surface`, taken apart by
/// `gfx::pixel_to_components`. A 16-bit pixel is read with `mov ax`: the upper half of eax
/// is the row's address's, and that is what the conversion gets.
pub fn read_pixel(x: u32, y: u32) -> Result<u32, BadInput> {
    let (_, pixel) = raw_pixel(x, y)?;
    let format = crate::gfx::pixel_format::layout(DRAW_SURFACE.format.get())?;
    Ok(crate::gfx::pixel_to_components::pixel_to_components(
        &format, pixel,
    ))
}

/// The row's address and the value `read_pixel` hands to the conversion (one place for
/// both: a second copy of this compiled without the row's upper half, 2026-10-16).
pub fn raw_pixel(x: u32, y: u32) -> Result<(u32, u32), BadInput> {
    let row = DRAW_SURFACE
        .pixels
        .get()
        .wrapping_add(y.wrapping_mul(DRAW_SURFACE.pitch.get()));
    let pixel = if DRAW_SURFACE.bits_per_pixel.get() == 0x10 {
        // As the original: eax the row's address, then `mov ax` of the pixel: the
        // upper half of the result is the row's address's.
        let word =
            GamePtr::new(row.wrapping_add(x.wrapping_mul(2))).read::<u16>("the surface's pixel")?;
        (row & 0xFFFF_0000) | u32::from(word)
    } else {
        GamePtr::new(row.wrapping_add(x.wrapping_mul(4))).read::<u32>("the surface's pixel")?
    };
    Ok((row, pixel))
}

/// `race::mud_pixel` (0x44A58A): `read_pixel` inside the surface (unsigned), else 0.
pub fn mud_pixel(x: u32, y: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::mud::inside`.
    if !rc2k_core::race::mud::inside(x, y, DRAW_SURFACE.width.get(), DRAW_SURFACE.height.get()) {
        return Ok(0);
    }
    read_pixel(x, y)
}

/// `race::load_mud` (0x449D95): the car's file number's two digits (`div 10`, each +
/// '0', as bytes) into the path, the file (0x10000 bytes at most) into the mud.
pub fn load_mud() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::file_digits`.
    let (tens, units) = rc2k_core::race::mud::file_digits(car_file_number(
        crate::race::blocks_store::own_car().car_kind.get(),
    ));
    RACE_VARS.mud_path.set(0x11, tens as u8);
    RACE_VARS.mud_path.set(0x12, units as u8);
    MUD.with(|mud| {
        crate::files::read_whole_file::read_whole_file_into(
            crate::pointers::GamePtr::to(&RACE_VARS.mud_path),
            mud,
        )
    })??;
    Ok(())
}

/// `race::clear_mud_state` (0x449E01): the 18 dwords at 0x623D58 and the two more 0.
pub fn clear_mud_state() {
    // The 18 dwords from `mud_frames` to the end of `mud_splash_counts`.
    let v = &RACE_VARS;
    v.mud_frames.set(0);
    v.mud_texture.set(0);
    for k in 0..4 {
        v.mud_thrown_in[k].set_bits(0);
        v.mud_splashes_in[k].set_bits(0);
        v.mud_thrown_counts[k].set(0);
        v.mud_splash_counts[k].set(0);
    }
    v.mud_turn_a[0].set(0);
    v.mud_turn_b[0].set(0);
}

/// `race::sample_mud_colours` (0x44A0D6): the surface locked (`d3d::lock_texture`), 256
/// colours, n's at (0x89 + (n & 3) + (n >> 5) * 4, 0x68 + ((n & 0x1C) >> 2)) by
/// `mud_pixel`, unlocked (`d3d::unlock_texture`).
pub fn sample_mud_colours() -> Result<(), Refused> {
    // dword_5D0DEC (`g_kindFirstTextures` + 0x304): the surface sampled.
    crate::d3d::lock_texture::lock_texture(KIND_TEXTURES.kind_first_textures[193].get())?;
    // Converted: `rc2k_core::race::mud::sample_place`.
    for n in 0..rc2k_core::race::mud::SAMPLES {
        let (x, y) = rc2k_core::race::mud::sample_place(n);
        RACE_VARS.mud_colours[n as usize].set(mud_pixel(x, y)?);
    }
    crate::d3d::unlock_texture::unlock_texture(KIND_TEXTURES.kind_first_textures[193].get())?;
    Ok(())
}

/// The mud file's strengths (`g_mud`, the game's 0x10000-byte block), Rust's since phase
/// 12: taken by `init_mud` counted on `stru_5C6F50`, given back by `race::free_mud`.
pub static MUD: crate::memory::owned::Owned<Vec<u8>> = crate::memory::owned::Owned::new("no mud");

/// `race::init_mud` (0x449DD6): the list's total + 0x10000, the mud taken (`MUD`, the
/// game's `dword_623D50`), then `load_mud`, `clear_mud_state`, `sample_mud_colours`.
pub fn init_mud() -> Result<(), Refused> {
    COURSE_LISTS
        .course_bytes_total
        .set(COURSE_LISTS.course_bytes_total.bits().wrapping_add(MUD_LEN));
    MUD.take(
        crate::pointers::GamePtr::to(&COURSE_LISTS.mud_list),
        MUD_LEN,
        vec![0; MUD_LEN as usize],
    )?;
    load_mud()?;
    clear_mud_state();
    sample_mud_colours()
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {

    /// The original block: `mov ax, [at]` over eax = the row's address.
    pub(super) unsafe fn mov_ax(row: u32, at: u32) -> u32 {
        let mut eax = row;
        unsafe {
            core::arch::asm!(
                "mov ax, word ptr [{at}]",
                at = in(reg) at,
                inout("eax") eax,
                options(nostack, readonly, preserves_flags),
            );
        }
        eax
    }

    fn mix(state: &mut u64) -> u32 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*state >> 32) as u32
    }

    #[test]
    fn mov_ax_matches() {
        let mut state = 7u64;
        let buffer = vec![0u8; 4096];
        let base = crate::pointers::game_address(buffer.as_ptr());
        let mut buffer = buffer;
        for b in buffer.iter_mut() {
            *b = mix(&mut state) as u8;
        }
        for _ in 0..5000 {
            let at = base + mix(&mut state) % 4000;
            let row = mix(&mut state);
            let want = unsafe { mov_ax(row, at) };
            let got = (row & 0xFFFF_0000)
                | u32::from(unsafe { (crate::pointers::host(at) as *const u16).read_unaligned() });
            assert_eq!(want, got);
        }
    }
}
