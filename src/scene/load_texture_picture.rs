//! `scene::load_texture_picture` (0x42578E): loads a numbered picture into a system
//! texture.
//!
//! In:  eax = the picture's number, ebx = the texture
//! Out: the original leaves eax, ebx, ecx, edx and ebp changed; its one caller keeps
//!      them all. The Rust version keeps them.
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
//! over 8, is a car's number under `dword_63DB90`): "carnum <n> grav <g>" is logged,
//! g being `dword_6235FC[car]`; with g at 3 or more and the file there, the picture
//! is taken from `files\b\gravel` instead, and its second name logged.
//!
//! Then `gfx::load_picture_to_texture` with eax = the name,
//! ecx = the second name, ebx = the texture.
//! One caller, `scene::load_changed_texture_picture`.
//! Shadow mode runs the original alone and compares the names it left with the Rust
//! version's.
//! Self-test: a redirected copy whose loading notes the names, 336 cases.
//!
//! Converted 2026-10-21: the digits, the names, the folder and the gravel rule through
//! `rc2k_core::scene::pictures`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::files::file_exists::file_exists;
use crate::framework::{game_addr, game_ptr, game_read, game_write, rva};
use crate::gfx::load_picture_to_texture::load_picture_to_texture_in_game;
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::scene::textures::KIND_TEXTURE_RVA;

// ---- game data ----

/// `dword_60AF04` the picture's number, `dword_60AF08` the texture, then the twelve
/// names up to the end of `files\s\extr\txxxxx.pcx`.
const WORK_RVA: usize = rva(0x60AF04);
/// The six folders' names and second names, and where the number is in the name.
const FOLDERS: [(usize, usize, usize); 6] = [
    (rva(0x60AF0C), rva(0x60AF8F), 0x0A), // files\a
    (rva(0x60AF1F), rva(0x60AFA2), 0x0A), // files\m
    (rva(0x60AF32), rva(0x60AFB5), 0x0A), // files\b
    (rva(0x60AF45), rva(0x60AFC8), 0x11), // files\b\gravel
    (rva(0x60AF5F), rva(0x60AFE2), 0x0F), // files\g\frnt
    (rva(0x60AF77), rva(0x60AFFA), 0x0F), // files\s\extr
];
use rc2k_core::scene::pictures::{GRAVEL, folder, names, picture_car, takes_gravel};
const FIRST_CAR_TEXTURE_RVA: usize = KIND_TEXTURE_RVA + 0x100 * 4; // dword_5D0EE8
const CARS_RVA: usize = rva(0x63DB90); // dword_63DB90
const CAR_GRAVEL_RVA: usize = rva(0x6235FC); // dword_6235FC
// gfx::load_picture_to_texture

/// What the function calls.
pub enum Call {
    Text(*const u8),
    Number(u32),
    /// Answers whether the file is there.
    Exists(usize),
    /// The name, the second name, the texture.
    Load(usize, usize, u32),
}

// ---- Rust version ----

/// The numbers and the names: everything the function writes.
unsafe fn write_names(number: u32, texture: u32) {
    unsafe {
        game_write(WORK_RVA, [number, texture]);
        for (&(name, second_name, at), (first, second)) in FOLDERS.iter().zip(names(number)) {
            (game_addr(name + at) as *mut [u8; 4]).write(first);
            if let Some(second) = second {
                (game_addr(second_name + at) as *mut [u8; 4]).write(second);
            }
        }
    }
}

// Converted: `rc2k_core::scene::pictures`
pub unsafe fn load_texture_picture(number: u32, texture: u32, call: &mut dyn FnMut(Call) -> bool) {
    unsafe {
        write_names(number, texture);
        let mut gravel_chosen = false;
        let car = picture_car(
            number,
            texture,
            || game_read(FIRST_CAR_TEXTURE_RVA),
            || game_read(CARS_RVA),
        );
        if let Some(car) = car {
            call(Call::Text(c"carnum".as_ptr().cast()));
            call(Call::Number(car));
            call(Call::Text(c"grav".as_ptr().cast()));
            let gravel: u32 = game_read(CAR_GRAVEL_RVA + car as usize * 4);
            call(Call::Number(gravel));
            if takes_gravel(gravel) && call(Call::Exists(FOLDERS[GRAVEL].0)) {
                gravel_chosen = true;
                call(Call::Text(game_ptr(FOLDERS[GRAVEL].1)));
            }
        }
        let (name, second, _) = FOLDERS[folder(number, gravel_chosen)];
        call(Call::Load(name, second, texture));
    }
}

/// With the game's log, files and loading.
pub unsafe fn load_texture_picture_in_game(number: u32, texture: u32) -> Result<(), Refused> {
    let mut refused = None;
    unsafe {
        load_texture_picture(number, texture, &mut |call| {
            match call {
                Call::Text(text) => log_write(text, false, 1),
                Call::Number(number) => log_int(number as i32, false, 1),
                Call::Exists(path) => match file_exists(game_ptr(path)) {
                    Ok(exists) => return exists,
                    Err(e) => {
                        refused.get_or_insert(e.into());
                    }
                },
                Call::Load(name, second, texture) => {
                    if let Err(e) = load_picture_to_texture_in_game(
                        game_addr(name) as u32,
                        game_addr(second) as u32,
                        texture,
                    ) {
                        refused.get_or_insert(e);
                    }
                }
            }
            false
        });
    }
    refused.map_or(Ok(()), Err)
}
