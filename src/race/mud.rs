//! `race::loadlevel_init`'s "render3d initmud": 64 KB on
//! `CourseLists.mud_list[0]` for the mud (`mud`), the car's mud file
//! (`files\b\mudfile.0xx`, xx the car's file number) read into it, the mud's state
//! cleared, and 256 colours sampled from a picture surface (`KindTextureTables.kind_first_textures[193]`, locked and
//! unlocked through DirectDraw: the game's, `d3d::lock_texture` / `d3d::unlock_texture`).

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::race::kind_store::KIND_TABLES;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

pub const MUD_LEN: u32 = 0x1_0000;

/// `race::car_file_number` (0x41F4FD): the car number held to 0 .. 0x21 (signed), its entry of `car_file_numbers`.
pub fn car_file_number(car: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    KIND_TABLES.car_file_numbers[n].get()
}

/// `gfx::read_pixel` (0x406584): the pixel at of `g_surface`, taken apart by
/// `gfx::pixel_to_components`. The original reads a 16-bit pixel into the low half of a
/// register that still holds the row's address, so the conversion is handed the row
/// address's upper half over the pixel; the conversion looks only at the bits its format's
/// masks name, which for a 16-bit surface are all in the low half, so the pixel alone is
/// handed here.
pub fn read_pixel(x: u32, y: u32) -> Result<u32, BadInput> {
    let drawing = &gfx().drawing;
    let rows = crate::gfx::rows::Rows::new(
        drawing.draw_surface.pixels.get(),
        drawing.draw_surface.pitch.get(),
        drawing.draw_surface.height.get(),
    );
    let row = rows.at(y, 0)?;
    // A pixel of the surface's row: bytes of a picture, not a record.
    let pixel = if drawing.draw_surface.bits_per_pixel.get() == 0x10 {
        u32::from(
            row.offset(x.wrapping_mul(2))
                .read::<u16>("the surface's pixel")?,
        )
    } else {
        row.offset(x.wrapping_mul(4))
            .read::<u32>("the surface's pixel")?
    };
    let format = crate::gfx::pixel_format::layout(drawing.draw_surface.format.get())?;
    Ok(crate::gfx::pixel_to_components::pixel_to_components(
        &format, pixel,
    ))
}

/// `race::mud_pixel` (0x44A58A): `read_pixel` inside the surface (unsigned), else 0.
pub fn mud_pixel(x: u32, y: u32) -> Result<u32, BadInput> {
    let drawing = &gfx().drawing;
    // Converted: `rc2k_core::race::mud::inside`.
    if !rc2k_core::race::mud::inside(
        x,
        y,
        drawing.draw_surface.width.get(),
        drawing.draw_surface.height.get(),
    ) {
        return Ok(0);
    }
    read_pixel(x, y)
}

/// `race::load_mud` (0x449D95): the car's file number's two digits (`div 10`, each + '0', as bytes)
/// into the path, the file (0x10000 bytes at most) into the mud.
pub fn load_mud(shared: &RaceShared) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::file_digits`.
    let (tens, units) = rc2k_core::race::mud::file_digits(car_file_number(
        crate::race::blocks_store::own_car().car_kind.get(),
    ));
    shared.vars.mud_path.set(0x11, tens as u8);
    shared.vars.mud_path.set(0x12, units as u8);
    MUD.with(|mud| {
        crate::files::read_whole_file::read_whole_file_into(&shared.vars.mud_path.text(), mud)
    })??;
    Ok(())
}

/// `race::clear_mud_state` (0x449E01): the 18 dwords at 0x623D58 and the two more 0.
pub fn clear_mud_state(shared: &RaceShared) {
    // The 18 dwords from `mud_frames` to the end of `mud_splash_counts`.
    let v = &shared.vars;
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

/// `race::sample_mud_colours` (0x44A0D6): the surface locked (`d3d::lock_texture`), 256 colours,
/// n's at (0x89 + (n & 3) + (n >> 5) * 4, 0x68 + ((n & 0x1C) >> 2)) by `mud_pixel`, unlocked
/// (`d3d::unlock_texture`).
pub fn sample_mud_colours(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let textures = &scene.textures;
    // KindTextureTables.kind_first_textures[193] (`g_kindFirstTextures` + 0x304): the surface sampled.
    crate::d3d::lock_texture::lock_texture(textures.kinds.kind_first_textures[193].get())?;
    // Converted: `rc2k_core::race::mud::sample_place`.
    for n in 0..rc2k_core::race::mud::SAMPLES {
        let (x, y) = rc2k_core::race::mud::sample_place(n);
        shared.vars.mud_colours[n as usize].set(mud_pixel(x, y)?);
    }
    crate::d3d::unlock_texture::unlock_texture(textures.kinds.kind_first_textures[193].get())?;
    Ok(())
}

/// The mud file's strengths (`g_mud`, the game's 0x10000-byte block), Rust's since phase
/// 12: taken by `init_mud` counted on `CourseLists.mud_list[0]`, given back by `race::free_mud`.
pub static MUD: crate::memory::owned::Owned<Vec<u8>> = crate::memory::owned::Owned::new("no mud");

/// `race::init_mud` (0x449DD6): the list's total + 0x10000, the mud taken (`MUD`, the game's
/// `mud`), then `load_mud`, `clear_mud_state`, `sample_mud_colours`.
pub fn init_mud(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    course
        .lists
        .course_bytes_total
        .set(course.lists.course_bytes_total.get().wrapping_add(MUD_LEN));
    MUD.take(&course.lists.mud_list, MUD_LEN, vec![0; MUD_LEN as usize])?;
    load_mud(&race.shared)?;
    clear_mud_state(&race.shared);
    sample_mud_colours(&race.shared, scene)
}
