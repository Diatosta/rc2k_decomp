//! The installer and uninstaller, and what the game knows about its installation. Started among
//! the files of the game's CD (or its download folder) with no installation present, the program
//! runs the installer's pages (`run_installer`) instead of the game. The module also reads and
//! writes the installation's records (`var\ral.zog`, `var\install.cfg`), the languages file
//! `lang.cfg` and the edition file `files\d\gfx\sku`, makes the menu shortcuts, and removes the
//! game again.

pub mod apply_config;
pub mod check_at_start;
pub mod check_clio_disc;
pub mod choose_language;
pub mod end_text_line;
#[cfg(unix)]
pub mod mac_bundle;
pub mod normalise_path;
pub mod outside;
pub mod owner;
pub mod portable;
pub mod read_languages;
pub mod read_record;
pub mod read_registry;
pub mod read_sku;
pub mod run_after_exit;
pub mod run_flags;
pub mod runs_from_source;

// The installer (`install::run_installer` and its tree). Helpers:
pub mod installer;
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
pub mod edit_key;
pub mod folder_key;
pub mod folder_page;
pub mod folders;
pub mod ignore_room;
pub mod install_clio;
pub mod install_files;
pub mod install_frames;
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
pub mod name_tables;
pub mod next_edit;
pub mod open_installed;
pub mod open_installer;
pub mod page_key;
pub mod page_links;
pub mod page_records;
pub mod place_pieces;
pub mod place_row;
pub mod register_lobby;
pub mod removal_page;
pub mod room_page;
pub mod run_installer;
pub mod run_page;
pub mod run_pages;
pub mod run_script;
pub mod script_name;
pub mod set_lobby_value;
pub mod shortcuts;
pub mod show_backdrop;
pub mod start_edit;
pub mod store;
pub mod uninstall;
pub mod unregister_lobby;
pub mod value_by_number;
pub mod value_down;
pub mod value_up;
pub mod vars;
pub mod write_record;
pub mod write_registry;
