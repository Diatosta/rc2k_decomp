//! `scene::load_texture_picture`: loads a numbered picture into a system texture.
//!
//! A picture is `tx<number>.pcx`, four decimal digits, and with it goes the picture numbered
//! one higher (by the look of it its transparency). The folder is by the number: `files\a`
//! under 900, `files\m` for 900 and 901, `files\a` up to 999, `files\b` up to 1999,
//! `files\g\frnt` up to 2999, else `files\s\extr`. The digits are written into all six names
//! and into five of the six second names (the `files\s\extr` one keeps its `xxxx`; and the
//! `files\m` name gets its third digit twice: slips of the 1999 code, kept).
//!
//! For a picture 1000 .. 1999 of a car (the texture less kind 0x100's first texture, over 8, is
//! a car's number under `RaceView.race_cars`): "carnum `<n>` grav `<g>`" is logged, g being
//! `RaceVars.car_tyres[0][car]`; with g at 3 or more and the file there, the picture is taken
//! from `files\b\gravel` instead, and its second name logged.
//!
//! Then the picture is read for the texture with the name and the second name
//! (`gfx::load_picture_to_texture::read_picture_for_texture`) and returned; the one caller,
//! `scene::load_changed_texture_picture`, uploads it.
//!
//! The digits, the names, the folder and the gravel rule are in the core crate
//! (`rc2k_core::scene::pictures`).

use crate::errors::{BadInput, Refused};
use crate::gfx::load_picture_to_texture::{TexturePicture, read_picture_for_texture};
use crate::logfile::log_int::log_int;
use crate::memory::place::BytePlace;

// ---- game data ----

/// The twelve names up to the end of `files\s\extr\txxxxx.pcx`. The six folders'
/// names and second names, and where the number is in the name.
fn folders(course: &'static crate::race::owner_course::Course) -> [(BytePlace, BytePlace, u32); 6] {
    let p = course.level_paths();
    [
        ((&p.picture_a).into(), (&p.second_a).into(), 0x0A), // files\a
        ((&p.picture_m).into(), (&p.second_m).into(), 0x0A), // files\m
        ((&p.picture_b).into(), (&p.second_b).into(), 0x0A), // files\b
        ((&p.picture_gravel).into(), (&p.second_gravel).into(), 0x11), // files\b\gravel
        ((&p.picture_front).into(), (&p.second_front).into(), 0x0F), // files\g\frnt
        ((&p.picture_extra).into(), (&p.second_extra).into(), 0x0F), // files\s\extr
    ]
}
use crate::logfile::log_write::log_nul;
use rc2k_core::scene::pictures::{GRAVEL, folder, names, picture_car, takes_gravel};

/// The six folders' picture names and second names with the picture's number in them.
fn write_names(
    course: &'static crate::race::owner_course::Course,
    number: u32,
) -> Result<(), BadInput> {
    for ((name, second_name, at), (first, second)) in folders(course).into_iter().zip(names(number))
    {
        name.add(at).write(&first, "a picture's file number")?;
        if let Some(second) = second {
            second_name
                .add(at)
                .write(&second, "a picture's file number")?;
        }
    }
    Ok(())
}

// Logic: `rc2k_core::scene::pictures`
pub fn load_texture_picture(
    race: &'static crate::race::owner::Race,
    gfx: &crate::gfx::owner::Gfx,
    number: u32,
    texture: u32,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<TexturePicture, Refused> {
    let (course, shared) = (&race.course, &race.shared);
    write_names(course, number)?;
    let mut gravel_chosen = false;
    let car = picture_car(
        number,
        texture,
        || scene.textures.first_textures()[0x100], // Textures::first_textures[256],
        || shared.view.get().race_cars.get(),
    );
    if let Some(car) = car {
        log_nul(b"carnum", false, 1);
        log_int(car as i32, false, 1);
        log_nul(b"grav", false, 1);
        // Eight cars' gravel switches; the car is under the race's cars, which every
        // write holds to eight (`init_texture_levels` holds them again at each stage).
        let gravel = shared
            .vars
            .get()
            .car_tyres
            .get(car as usize)
            .map(crate::memory::session::PlainDword::get)
            .ok_or(BadInput("a car past the eight tyre switches"))?;
        log_int(gravel as i32, false, 1);
        if takes_gravel(gravel)
            && crate::files::file_exists::file_exists(
                &folders(course)[GRAVEL].0.text("a texture folder")?,
            )?
        {
            gravel_chosen = true;
            log_nul(
                &folders(course)[GRAVEL].1.text("a folder's name")?[..],
                false,
                1,
            );
        }
    }
    let (name, second, _) = folders(course)[folder(number, gravel_chosen)];
    read_picture_for_texture(Some(&name), Some(&second), texture, gfx, system)
}
