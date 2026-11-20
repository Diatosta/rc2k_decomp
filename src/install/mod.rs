//! The installer and uninstaller, and what the game knows about its installation. Started among
//! the files of the game's CD (or its download folder) with no installation present, the program
//! runs the installer's pages (`run_installer`) instead of the game. The module also reads and
//! writes the installation's records (`var\ral.zog`, `var\install.cfg`), the languages file
//! `lang.cfg` and the edition file `files\d\gfx\sku`, makes the menu shortcuts, and removes the
//! game again.

pub(crate) mod apply_config;
pub(crate) mod check_at_start;
mod choose_language;
mod end_text_line;
pub(crate) mod find_clio;
#[cfg(unix)]
mod mac_bundle;
mod normalise_path;
pub(crate) mod outside;
pub(crate) mod owner;
pub(crate) mod read_languages;
pub(crate) mod read_record;
pub(crate) mod read_sku;
pub(crate) mod record;
pub(crate) mod run_after_exit;
pub(crate) mod run_flags;
pub(crate) mod runs_from_source;

// The installer (`install::run_installer` and its tree). Helpers:
mod installer;
// The list-menu pages:
mod backdrop_frame;
mod blit_glyph_list;
mod check_folder_name;
mod check_installed_version;
mod close_installer;
mod copy_counted;
mod draw_pieces;
mod draw_string_list;
mod folders;
mod install_files;
mod install_frames;
mod lay_out_pieces;
mod list_drives;
mod list_files;
mod list_folder;
mod list_folder_tree;
mod made_texts;
mod make_folders;
mod measure_pieces;
mod name_tables;
mod open_installed;
mod open_installer;
mod page_actions;
mod page_key;
mod page_links;
mod page_state;
mod place_pieces;
mod place_row;
pub(crate) mod run_installer;
mod run_page;
mod run_pages;
mod run_script;
mod shortcuts;
mod show_backdrop;
mod store;
mod uninstall;
mod vars;
mod write_record;
