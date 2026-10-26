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

use crate::pointers::GamePtr;
use crate::scene::file_numbers_store::{FILE_NUMBERS, texts};
use crate::scene::object_store::OBJECT_WORK;

// ---- game data ----

/// For each folder (`rc2k_core::scene::objects::folder`): the `.ro3` path, the `.lwo`
/// path, and how far in the number is.
const FOLDERS: [(usize, usize, usize); 4] = [
    (texts::FOLDER_0_RO3, texts::FOLDER_0_LWO, 0x0B),
    (texts::FOLDER_1_RO3, texts::FOLDER_1_LWO, 0x0B),
    (texts::FOLDER_2_RO3, texts::FOLDER_2_LWO, 0x10),
    (texts::FOLDER_3_RO3, texts::FOLDER_3_LWO, 0x10),
];

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::{folder, number_digits}`
pub fn set_object_file_names(folder_kind: u32) -> Result<(), crate::errors::BadInput> {
    use rc2k_core::scene::objects::{folder, number_digits};
    {
        let (ro3, lwo, number) = FOLDERS[folder(folder_kind)];
        let (ro3_number, lwo_number) = (
            FILE_NUMBERS.text_addr(ro3 + number),
            FILE_NUMBERS.text_addr(lwo + number),
        );
        FILE_NUMBERS.object_ro3_number.set_bits(ro3_number as u32);
        FILE_NUMBERS
            .object_ro3_path
            .set_bits(FILE_NUMBERS.text_addr(ro3) as u32);
        FILE_NUMBERS
            .object_lwo_path
            .set_bits(FILE_NUMBERS.text_addr(lwo) as u32);
        FILE_NUMBERS.object_lwo_number.set_bits(lwo_number as u32);
        let digits = number_digits(OBJECT_WORK.loading_kind.bits());
        GamePtr::new(ro3_number as u32).write(digits, "the object's file number")?;
        GamePtr::new(lwo_number as u32).write(digits, "the object's file number")?;
        Ok(())
    }
}
