//! `scene::set_object_file_names` (0x43964A): the paths of a kind of object's two
//! files.
//!
//! In:  eax = the kind (which folder), `dword_634FE8` = the kind (the number)
//! Out: eax = the tens digit as a character, ecx = 10, edx = the units digit as a
//!      character, esi and ebp = where the number is in the two names
//!
//! By the kind: under 0x190 `files\a\obj????.ro3` and `.lwo`, under 0x258 `files\b\`, under
//! 0x320 `files\g\frnt\`, else `files\s\extr\`. Four pointers are set: the `.ro3` path
//! (`dword_60AD28`) and where its number is (`dword_60AD24`), the `.lwo` path (`dword_60AD2C`)
//! and its number (`dword_60AD30`). The number's four decimal digits are written over the
//! `????` of both (the thousands digit as one byte: anything above 9999 gives another
//! character). One caller, `scene::load_object`.

use crate::framework::{game_addr, game_read, game_write, rva};

// ---- game data ----

/// `dword_60AD24` the `.ro3` name's number, `dword_60AD28` the `.ro3` path,
/// `dword_60AD2C` the `.lwo` path, `dword_60AD30` the `.lwo` name's number.
pub const NAMES_RVA: usize = rva(0x60AD24);
pub const KIND_RVA: usize = rva(0x634FE8); // dword_634FE8
/// For each folder (`rc2k_core::scene::objects::folder`): the `.ro3` path, the `.lwo`
/// path, and how far in the number is.
const FOLDERS: [(usize, usize, usize); 4] = [
    (rva(0x60AD4E), rva(0x60AD62), 0x0B),
    (rva(0x60AD76), rva(0x60AD8A), 0x0B),
    (rva(0x60AD9E), rva(0x60ADB7), 0x10),
    (rva(0x60ADD0), rva(0x60ADE9), 0x10),
];

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::{folder, number_digits}`
pub unsafe fn set_object_file_names(folder_kind: u32) {
    use rc2k_core::scene::objects::{folder, number_digits};
    unsafe {
        let (ro3, lwo, number) = FOLDERS[folder(folder_kind)];
        let (ro3_number, lwo_number) = (game_addr(ro3 + number), game_addr(lwo + number));
        game_write(
            NAMES_RVA,
            [
                ro3_number as u32,
                game_addr(ro3) as u32,
                game_addr(lwo) as u32,
                lwo_number as u32,
            ],
        );
        let digits = number_digits(game_read(KIND_RVA));
        (ro3_number as *mut [u8; 4]).write(digits);
        (lwo_number as *mut [u8; 4]).write(digits);
    }
}
