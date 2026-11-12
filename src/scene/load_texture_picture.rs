//! `scene::load_texture_picture` (0x42578E): loads a numbered picture into a system
//! texture.
//!
//! Takes: the picture's number, the texture
//!
//! A picture is `tx<number>.pcx`, four decimal digits, and with it goes the picture
//! numbered one higher (by the look of it its transparency). The folder is by the
//! number: `files\a` under 900, `files\m` for 900 and 901, `files\a` up to 999,
//! `files\b` up to 1999, `files\g\frnt` up to 2999, else `files\s\extr`. The digits
//! are written into all six names and into five of the six second names (the
//! `files\s\extr` one keeps its `xxxx`; and the `files\m` name gets its third digit
//! twice: the original's slips, kept).
//!
//! For a picture 1000 .. 1999 of a car (the texture less kind 0x100's first texture,
//! over 8, is a car's number under `RaceView.race_cars`): "carnum <n> grav <g>" is logged,
//! g being `RaceVars.car_tyres[0][car]`; with g at 3 or more and the file there, the picture
//! is taken from `files\b\gravel` instead, and its second name logged.
//!
//! Then `gfx::load_picture_to_texture` with the name, the second name and the
//! texture. One caller, `scene::load_changed_texture_picture`.
//!
//! The logic is in the core crate: the digits, the names, the folder and the gravel rule through
//! `rc2k_core::scene::pictures`.

use crate::errors::{BadInput, Refused};
use crate::gfx::load_picture_to_texture::load_picture_to_texture_in_game;
use crate::logfile::log_int::log_int;
use crate::place::BytePlace;

// ---- game data ----

/// `ObjectFileNumbers.picture_number` the picture's number, `ObjectFileNumbers.picture_texture` the texture, then the twelve
/// names up to the end of `files\s\extr\txxxxx.pcx`.
/// The six folders' names and second names, and where the number is in the name.
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
use rc2k_core::scene::pictures::{GRAVEL, folder, names, picture_car, takes_gravel};
// gfx::load_picture_to_texture

/// What the function calls.
pub enum Call {
    Text(Vec<u8>),
    Number(u32),
    /// Answers whether the file is there.
    Exists(BytePlace),
    /// The name, the second name, the texture.
    Load(BytePlace, BytePlace, u32),
}

// ---- Rust version ----

/// The numbers and the names: everything the function writes.
fn write_names(
    course: &'static crate::race::owner_course::Course,
    number: u32,
    texture: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let file_numbers = &scene.file_numbers();
    {
        file_numbers.picture_number.set(number);
        file_numbers.picture_texture.set(texture);
        for ((name, second_name, at), (first, second)) in
            folders(course).into_iter().zip(names(number))
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
}

// Converted: `rc2k_core::scene::pictures`
pub fn load_texture_picture(
    race: &'static crate::race::owner::Race,
    number: u32,
    texture: u32,
    call: &mut dyn FnMut(Call) -> bool,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let (course, shared) = (&race.course, &race.shared);
    {
        write_names(course, number, texture, scene)?;
        let mut gravel_chosen = false;
        let car = picture_car(
            number,
            texture,
            || scene.textures.kinds.get().kind_first_textures[0x100].get(), // KindTextureTables.kind_first_textures[256],
            || shared.view.get().race_cars.get(),
        );
        if let Some(car) = car {
            call(Call::Text(b"carnum".to_vec()));
            call(Call::Number(car));
            call(Call::Text(b"grav".to_vec()));
            // Eight cars' gravel switches; a car past them (the race's cars are at most
            // eight) reads on, as the original.
            let tyres = &shared.vars.get().car_tyres;
            let gravel = crate::session::read_on_in(
                shared.vars.get(),
                tyres,
                car as usize,
                "a car's tyre switch",
            )?;
            call(Call::Number(gravel));
            if takes_gravel(gravel) && call(Call::Exists(folders(course)[GRAVEL].0)) {
                gravel_chosen = true;
                call(Call::Text(
                    folders(course)[GRAVEL].1.text("a folder's name")?,
                ));
            }
        }
        let (name, second, _) = folders(course)[folder(number, gravel_chosen)];
        call(Call::Load(name, second, texture));
        Ok(())
    }
}

/// With the game's log, files and loading.
pub fn load_texture_picture_in_game(
    race: &'static crate::race::owner::Race,
    gfx: &crate::gfx::owner::Gfx,
    number: u32,
    texture: u32,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let mut refused = None;
    load_texture_picture(
        race,
        number,
        texture,
        &mut |call| {
            match call {
                Call::Text(text) => crate::logfile::log_write::log_nul(&text[..], false, 1),
                Call::Number(number) => log_int(number as i32, false, 1),
                Call::Exists(path) => {
                    match path
                        .text("a texture folder")
                        .and_then(|p| crate::files::file_exists::file_exists(&p))
                    {
                        Ok(exists) => return exists,
                        Err(e) => {
                            refused.get_or_insert(e.into());
                        }
                    }
                }
                Call::Load(name, second, texture) => {
                    if let Err(e) = load_picture_to_texture_in_game(
                        Some(&name),
                        Some(&second),
                        texture,
                        gfx,
                        system,
                    ) {
                        refused.get_or_insert(e);
                    }
                }
            }
            false
        },
        scene,
    )?;
    refused.map_or(Ok(()), Err)
}
