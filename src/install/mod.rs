//! The installer and uninstaller, and what the game knows about its installation. Started among
//! the files of the game's CD (or its download folder) with no installation present, the program
//! runs the installer's pages (`run_installer`) instead of the game. The module also reads and
//! writes the installation's records (`var\ral.zog`, `var\install.cfg`), the languages file
//! `lang.cfg` and the edition file `files\d\gfx\sku`, makes the menu shortcuts, and removes the
//! game again.

pub(crate) mod apply_config;
pub(crate) mod check_at_start;
pub(crate) mod check_clio_disc;
mod choose_language;
mod end_text_line;
#[cfg(unix)]
mod mac_bundle;
mod normalise_path;
pub(crate) mod outside;
pub(crate) mod owner;
pub(crate) mod portable;
pub(crate) mod read_languages;
pub(crate) mod read_record;
mod read_registry;
pub(crate) mod read_sku;
pub(crate) mod run_after_exit;
pub(crate) mod run_flags;
pub(crate) mod runs_from_source;

// The installer (`install::run_installer` and its tree). Helpers:
mod installer;
// The list-menu pages:
mod ask_registration;
mod backdrop_frame;
mod blit_glyph_list;
mod check_folder_name;
mod check_installed_version;
mod choice_back;
mod choice_by_number;
mod choice_on;
mod close_installer;
mod copy_counted;
mod draw_pieces;
mod draw_string_list;
mod drive_page;
mod edit_key;
mod folder_key;
mod folder_page;
mod folders;
mod ignore_room;
mod install_clio;
mod install_files;
mod install_frames;
mod install_from_command;
mod item_action;
mod item_enter;
mod item_link;
mod key_down;
mod key_left;
mod key_number;
mod key_right;
mod key_up;
mod language_page;
mod lay_out_pieces;
mod list_drives;
mod list_files;
mod list_folder;
mod list_folder_tree;
mod list_pages;
mod made_texts;
mod make_folders;
mod mark_main_page;
mod mark_page;
mod measure_pieces;
mod name_tables;
mod next_edit;
mod open_installed;
mod open_installer;
mod page_key;
mod page_links;
mod page_records;
mod place_pieces;
mod place_row;
mod register_lobby;
mod removal_page;
mod room_page;
pub(crate) mod run_installer;
mod run_page;
mod run_pages;
mod run_script;
mod script_name;
mod set_lobby_value;
mod shortcuts;
mod show_backdrop;
mod start_edit;
mod store;
mod uninstall;
mod unregister_lobby;
mod value_by_number;
mod value_down;
mod value_up;
mod vars;
mod write_record;
mod write_registry;
