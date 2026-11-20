//! `scene::set_object_file_names`: the paths of a kind of object's two files. Takes the kind
//! (which folder); `ObjectWork.loading_kind` = the kind (the number).
//!
//! By the kind: under 0x190 `files\a\obj????.ro3` and `.lwo`, under 0x258 `files\b\`, under 0x320
//! `files\g\frnt\`, else `files\s\extr\`. Four pointers are set: the `.ro3` path
//! (`ObjectFileNumbers.object_ro3_path`) and where its number is
//! (`ObjectFileNumbers.object_ro3_number`), the `.lwo` path (`ObjectFileNumbers.object_lwo_path`)
//! and its number (`ObjectFileNumbers.object_lwo_number`). The number's four decimal digits are
//! written over the `????` of both (the thousands digit as one byte: anything above 9999 gives
//! another character).

use crate::memory::session::TextPlace;
use crate::scene::file_numbers_store::texts;

// ---- game data ----

/// For each folder (`rc2k_core::scene::objects::folder`): the `.ro3` path, the `.lwo`
/// path, and how far in the number is.
const FOLDERS: [(usize, usize, usize); 4] = [
    (texts::FOLDER_0_RO3, texts::FOLDER_0_LWO, 0x0B),
    (texts::FOLDER_1_RO3, texts::FOLDER_1_LWO, 0x0B),
    (texts::FOLDER_2_RO3, texts::FOLDER_2_LWO, 0x10),
    (texts::FOLDER_3_RO3, texts::FOLDER_3_LWO, 0x10),
];

// Logic: `rc2k_core::scene::objects::{folder, number_digits}`
pub fn set_object_file_names(
    scene: &crate::scene::owner::Scene,
    folder_kind: u32,
) -> Result<(), crate::errors::BadInput> {
    use rc2k_core::scene::objects::{folder, number_digits};
    {
        let (ro3, lwo, number) = FOLDERS[folder(folder_kind)];
        let (ro3_number, lwo_number) = ((ro3 + number) as u32, (lwo + number) as u32);
        scene.file_numbers_mut().object_ro3_number = ro3_number;
        scene.file_numbers_mut().object_ro3_path = ro3 as u32;
        scene.file_numbers_mut().object_lwo_path = lwo as u32;
        scene.file_numbers_mut().object_lwo_number = lwo_number;
        let digits = number_digits(scene.object_work().loading_kind);
        scene
            .path_text(ro3 + number)
            .write_text_at(0, &digits, "the object's file number")?;
        scene
            .path_text(lwo + number)
            .write_text_at(0, &digits, "the object's file number")?;
        Ok(())
    }
}
