# Module: install

*Category: [Module reference](Modules)*

The installer and uninstaller, and what the game knows about its installation. Started among
the files of the game's CD with no installation present, the program runs the installer's
pages (language, folder, copying, finish) instead of the game. The module also reads and writes
the installation's record, makes the menu shortcuts, and removes the game again.

## State

The owner is `Install` (`src/install/owner.rs`). Its parts: the installer's variables
(`src/install/vars.rs`, `store.rs`), the lists of names it builds (`name_tables.rs`) and the
pages' records (`page_records.rs`); the page being shown is a plain list of pieces, `Install::pieces`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `run_installer` | `src/install/run_installer.rs` | The installer from start to end. |
| `run_pages`, `run_page` | `src/install/run_pages.rs`, `src/install/run_page.rs` | Runs the flow's steps (`rc2k_core/src/install/flow.rs` says where each answer leads); runs one page until it is left. |
| `install_files` | `src/install/install_files.rs` | Copies the game's files, showing progress. |
| `list_files` | `src/install/list_files.rs` | Builds the list of what to copy. |
| `uninstall` | `src/install/uninstall.rs` | Removes the game (all of it, or keeping the saved games). |
| `is_source_folder` | `src/install/portable.rs` | Tells whether the program was started among the CD's files; this file also holds the portable record `var/install.cfg`. |
| `read_record`, `write_record` | files of the same names | The original's installation record. |
| `read_languages`, `read_sku` | files of the same names | The languages file and the edition file. |
| `check_at_start` | `src/install/check_at_start.rs` | The check of the installation made when the game starts. |
| `add_start_menu_items`, `remove_start_menu_items` | `src/install/shortcuts.rs` | The menu entries, per system. |
| `Folders` | `src/install/folders/mod.rs` | Where things go on each system (`windows.rs`, `unix.rs`). |
| `run_script` | `src/install/run_script.rs` | Runs the install script's steps. |
| `install_from_command` | `src/install/install_from_command.rs` | Installing asked for by a command-line switch. |

## How it is laid out

One function per file. Groups: `*_page.rs` (a page each), `key_*`, `value_*`, `choice_*` and
`item_*` (how a page's items react to keys), `*_pieces.rs` and `draw_*` (page layout),
`read_*` / `write_*` (records), `folders/` (per system).

## Calls and callers

It uses [`gfx`](Module-gfx) to draw, [`files`](Module-files) to copy,
[`strings`](Module-strings) for texts and the menu's list handling from [`menu`](Module-menu).
It is entered from `system::main_loop` and from start-up (`src/standalone.rs`).

## Related pages

[The installer](The-installer), [Install records](Install-records),
[Installer data](Installer-data).
