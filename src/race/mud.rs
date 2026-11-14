//! `race::loadlevel_init`'s "render3d initmud": 64 KB on `CourseLists.mud_list[0]` for the mud
//! (`mud`), the car's mud file (`files\b\mudfile.0xx`, xx the car's file number) read into it, the
//! mud's state cleared, and 256 colours sampled from a picture surface
//! (`KindTextureTables.kind_first_textures[193]`, locked and unlocked by `d3d::lock_texture` /
//! `d3d::unlock_texture`).

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::race::kind_store::KindTables;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::net::net_race::NetRace;

pub const MUD_LEN: u32 = 0x1_0000;

/// The car number held to 0 .. 0x21 (signed), its entry of `car_file_numbers`.
pub fn car_file_number(kinds: &KindTables, car: u32) -> u32 {
    // Logic in `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    kinds.car_file_numbers[n].get()
}

/// The pixel at (x, y) of `surface`, taken apart by `gfx::pixel_to_components`. The conversion
/// looks only at the bits its format's masks name, which for a 16-bit surface are all in the low
/// half, so the pixel alone is handed to it.
pub fn read_pixel(
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<u32, BadInput> {
    let drawing = &gfx.drawing;
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
    let format = crate::gfx::pixel_format::layout(system, gfx, drawing.draw_surface.format.get())?;
    Ok(crate::gfx::pixel_to_components::pixel_to_components(
        &format, pixel,
    ))
}

/// `read_pixel` inside the surface (unsigned), else 0.
pub fn mud_pixel(
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<u32, BadInput> {
    let drawing = &gfx.drawing;
    // Logic in `rc2k_core::race::mud::inside`.
    if !rc2k_core::race::mud::inside(
        x,
        y,
        drawing.draw_surface.width.get(),
        drawing.draw_surface.height.get(),
    ) {
        return Ok(0);
    }
    read_pixel(x, y, gfx, system)
}

/// Writes the car's file number's two digits (tens and units, each + '0') into the path and reads
/// the file (0x10000 bytes at most) into the mud.
pub fn load_mud(
    kinds: &KindTables,
    shared: &RaceShared,
    net_race: Option<NetRace>,
) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::mud::file_digits`.
    let (tens, units) = rc2k_core::race::mud::file_digits(car_file_number(
        kinds,
        shared.own_car(net_race).car_kind.get(),
    ));
    shared.vars.get().mud_path.set(0x11, tens as u8);
    shared.vars.get().mud_path.set(0x12, units as u8);
    MUD.with(|mud| {
        crate::files::read_whole_file::read_whole_file_into(&shared.vars.get().mud_path.text(), mud)
    })??;
    Ok(())
}

/// Zeroes the 18 mud dwords and the two after them.
pub fn clear_mud_state(shared: &RaceShared) {
    // The 18 dwords from `mud_frames` to the end of `mud_splash_counts`.
    let v = shared.vars.get();
    v.mud_frames.set(0);
    v.mud_texture.set(0);
    for k in 0..4 {
        v.mud_thrown_in[k].set(0.0);
        v.mud_splashes_in[k].set(0.0);
        v.mud_thrown_counts[k].set(0);
        v.mud_splash_counts[k].set(0);
    }
    v.mud_turn_a[0].set(0);
    v.mud_turn_b[0].set(0);
}

/// Samples 256 colours from the locked surface (`d3d::lock_texture`): colour n at
/// (0x89 + (n & 3) + (n >> 5) * 4, 0x68 + ((n & 0x1C) >> 2)) by `mud_pixel`; then unlocks it
/// (`d3d::unlock_texture`).
pub fn sample_mud_colours(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let textures = &scene.textures;
    // `KindTextureTables.kind_first_textures[193]`: the surface sampled.
    crate::d3d::lock_texture::lock_texture_in(
        gfx,
        textures.kinds.get().kind_first_textures[193].get(),
    )?;
    // Logic in `rc2k_core::race::mud::sample_place`.
    for n in 0..rc2k_core::race::mud::SAMPLES {
        let (x, y) = rc2k_core::race::mud::sample_place(n);
        shared.vars.get().mud_colours[n as usize].set(mud_pixel(x, y, gfx, system)?);
    }
    crate::d3d::unlock_texture::unlock_texture(
        textures.kinds.get().kind_first_textures[193].get(),
    )?;
    Ok(())
}

/// The mud file's strengths (`mud`, a 0x10000-byte block): taken by `init_mud` counted on
/// `CourseLists.mud_list[0]`, given back when the stage is freed.
pub static MUD: crate::memory::owned::Owned<Vec<u8>> = crate::memory::owned::Owned::new("no mud");

/// Adds 0x10000 to the list's total, takes the mud (`MUD`), then runs `load_mud`,
/// `clear_mud_state` and `sample_mud_colours`.
pub fn init_mud(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let course = &race.course;
    course.lists.get().course_bytes_total.set(
        course
            .lists
            .get()
            .course_bytes_total
            .get()
            .wrapping_add(MUD_LEN),
    );
    MUD.take(
        &course.lists.get().mud_list,
        MUD_LEN,
        vec![0; MUD_LEN as usize],
    )?;
    load_mud(race.kind_tables(), &race.shared, race.shared.net_race.get())?;
    clear_mud_state(&race.shared);
    sample_mud_colours(&race.shared, scene, gfx, system)
}
