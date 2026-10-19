//! What the game reads about its installation: the record `var\ral.zog`, the languages
//! file `lang.cfg`, the edition file `files\d\gfx\sku`.

use crate::framework::{GameHook, game_addr, rva};
use crate::redirect::RedirectedCopy;

pub mod check_at_start;
pub mod check_clio_disc;
pub mod check_test;
pub mod choose_language;
pub mod normalise_path;
pub mod outside;
pub mod read_languages;
pub mod read_record;
pub mod read_registry;
pub mod read_sku;
pub mod run_after_exit;
pub mod runs_from_source;
pub mod stand_ins;

// The installer (`sub_412DE0` and its tree). Helpers with no hook:
pub mod fakefs;
pub mod installer;
pub mod item_test;
pub mod list_test;
pub mod page_action_test;
// The list-menu pages:
pub mod ask_registration;
pub mod backdrop_frame;
pub mod blit_glyph_list;
pub mod check_folder_name;
pub mod check_installed_version;
pub mod choice_back;
pub mod choice_by_number;
pub mod choice_on;
pub mod close_installer;
pub mod copy_counted;
pub mod draw_pieces;
pub mod draw_string_list;
pub mod drive_page;
pub mod dxsetup_page;
pub mod edit_key;
pub mod finish_page;
pub mod folder_key;
pub mod folder_page;
pub mod ignore_room;
pub mod install_clio;
pub mod install_files;
pub mod install_from_command;
pub mod item_action;
pub mod item_enter;
pub mod item_link;
pub mod key_down;
pub mod key_left;
pub mod key_number;
pub mod key_right;
pub mod key_up;
pub mod language_page;
pub mod lay_out_pieces;
pub mod list_drives;
pub mod list_files;
pub mod list_folder;
pub mod list_folder_tree;
pub mod make_folders;
pub mod mark_main_page;
pub mod mark_page;
pub mod measure_pieces;
pub mod mode_page;
pub mod next_edit;
pub mod open_installed;
pub mod open_installer;
pub mod page_key;
pub mod place_pieces;
pub mod place_row;
pub mod register_lobby;
pub mod room_page;
pub mod run_installer;
pub mod run_page;
pub mod run_pages;
pub mod run_script;
pub mod script_name;
pub mod set_lobby_value;
pub mod show_backdrop;
pub mod start_edit;
pub mod uninstall;
pub mod unregister_lobby;
pub mod value_by_number;
pub mod value_down;
pub mod value_up;
pub mod write_record;
pub mod write_registry;

pub static HOOKS: &[&dyn GameHook] = &[
    &read_record::HOOK,
    &choose_language::HOOK,
    &read_languages::HOOK,
    &read_sku::HOOK,
    &normalise_path::HOOK,
    &read_registry::HOOK,
    &runs_from_source::HOOK,
    &check_at_start::HOOK,
    &check_clio_disc::HOOK,
    &run_after_exit::HOOK,
    &ask_registration::HOOK,
    &backdrop_frame::HOOK,
    &blit_glyph_list::HOOK,
    &check_folder_name::HOOK,
    &check_installed_version::HOOK,
    &choice_back::HOOK,
    &choice_by_number::HOOK,
    &choice_on::HOOK,
    &close_installer::HOOK,
    &copy_counted::HOOK,
    &draw_pieces::HOOK,
    &draw_string_list::HOOK,
    &drive_page::HOOK,
    &dxsetup_page::HOOK,
    &edit_key::HOOK,
    &finish_page::HOOK,
    &folder_key::HOOK,
    &folder_page::HOOK,
    &ignore_room::HOOK,
    &install_files::HOOK,
    &install_clio::HOOK,
    &uninstall::HOOK,
    &install_from_command::HOOK,
    &item_action::HOOK,
    &item_enter::HOOK,
    &item_link::HOOK,
    &key_down::HOOK,
    &key_left::HOOK,
    &key_number::HOOK,
    &key_right::HOOK,
    &key_up::HOOK,
    &language_page::HOOK,
    &lay_out_pieces::HOOK,
    &list_drives::HOOK,
    &list_folder::HOOK,
    &list_files::HOOK,
    &list_folder_tree::HOOK,
    &run_script::HOOK,
    &make_folders::HOOK,
    &mark_main_page::HOOK,
    &mark_page::HOOK,
    &measure_pieces::HOOK,
    &mode_page::HOOK,
    &next_edit::HOOK,
    &open_installed::HOOK,
    &open_installer::HOOK,
    &page_key::HOOK,
    &place_pieces::HOOK,
    &place_row::HOOK,
    &register_lobby::HOOK,
    &room_page::HOOK,
    &run_installer::HOOK,
    &run_page::HOOK,
    &run_pages::HOOK,
    &script_name::HOOK,
    &set_lobby_value::HOOK,
    &show_backdrop::HOOK,
    &start_edit::HOOK,
    &unregister_lobby::HOOK,
    &value_by_number::HOOK,
    &value_down::HOOK,
    &value_up::HOOK,
    &write_record::HOOK,
    &write_registry::HOOK,
];

/// byte_5189A8: base directory A (`cdrom` in the record), and byte_518AA8: what a `%` in a
/// path stands for (the language's directory). 0x100 bytes each.
pub const BASE_DIRECTORY_RVA: usize = rva(0x5189A8);
pub const LOCALE_RVA: usize = rva(0x518AA8);
pub const PATH_ROOM: usize = 0x100;

/// For the self-tests of the three readers: a copy of the original that reads the file
/// at `path` and not its own. All three start `pusha; lea eax, <path>`.
pub unsafe fn copy_reading(
    original: usize,
    len: usize,
    own_path: usize,
    calls: &[usize],
    path: &'static [u8],
) -> Result<RedirectedCopy, &'static str> {
    unsafe {
        let named = (game_addr(own_path) as u32).to_le_bytes();
        let entry = [
            0x60, 0x8D, 0x05, named[0], named[1], named[2], named[3], 0xE8,
        ];
        let copy = RedirectedCopy::new(game_addr(original), len, &entry, calls, &|_| None)?;
        copy.patch_u32(3, path.as_ptr() as u32);
        Ok(copy)
    }
}
