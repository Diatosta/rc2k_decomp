# The installer

*Category: [Game flow](Game-flow)*

The 1999 game was installed by its own program: `RAL.EXE` on the CD showed the installer's
pages instead of the game. This project keeps that. `rc2k`, started among the CD's files with
no installation present, becomes the installer; started in an installed folder, it is the
game. This page follows an installation and a removal through the code, and lists what was
deliberately changed.

For the player's instructions, see [Installing the game](Installing-the-game). For the files
involved, see [Install records](Install-records) and [Installer data](Installer-data).

The code is `src/install/`, with rules in `rc2k_core/src/install/`. The state is the `Install`
owner (`src/install/owner.rs`).

## Deciding to install

Start-up reads two things in `read_command_line_and_install_state` (`src/standalone.rs`): the
command line (`system::read_command_line`) and whatever record of an installation exists
(`src/install/read_record.rs`, `src/install/portable.rs`).

`system::main_loop` then runs the installer instead of the front end if any of these holds:

| Condition | Meaning | Checked by |
| --- | --- | --- |
| The folder has the game's data but no installation record | The program was started in the CD's files (or a copy of them). | `install::portable::is_source_folder` |
| The command line has `-u` | Uninstall the installation in this folder. | `install.flags.updating` |
| The installer was set up by start-up | The original's other routes in (`-i`, special discs). | `src/install/check_at_start.rs` |

"The CD's files" means a folder holding the disc's contents, with its `FILES` folder. It does
not have to be a CD: a copy on the hard disk works the same, and that is the expected way to
install today.

## The installer's run

`install::run_installer` (`src/install/run_installer.rs`):

1. reads which languages the CD has (`src/install/read_languages.rs`);
2. builds the installer's fonts and opens it: memory for the lists of folders and files, the
   installer's palette and progress pictures, the backdrop picture
   (`src/install/open_installer.rs`);
3. works out a default folder for this system;
4. checks whether this folder already holds an installation of the same build
   (`src/install/check_installed_version.rs`);
5. runs the flow of pages and work (`src/install/run_pages.rs`);
6. closes the installer and frees its fonts.

When it returns, the main loop asks the program to quit. What the pages asked to happen
*after* the program closes (shortcuts, starting the installed copy) is done on the way out by
`src/install/run_after_exit.rs`.

## Pages

A page is a **list menu**: a title and a column of items, each of which leaves the page with an
answer, does something, is a choice between values, or is a line of text to edit. The pages'
lists are typed records in `src/install/vars.rs`; the record layout is described in
`src/memory/list_handle.rs` (the layout) and `src/install/list_pages.rs` (the installer's lists and their links).

`install::run_page` (`src/install/run_page.rs`) runs one page until it is left. Each frame it
lays the page out, draws it over the backdrop, and takes a key. The key is offered first to
the highlighted item (`src/install/page_key.rs`), then to the page, and otherwise moves the
highlight. Each page has a small function that runs when the page opens, after each key and
when it closes; those are the `..._page.rs` files, called by name from `run_page.rs`.

| Page | File | What it does |
| --- | --- | --- |
| Main | `mark_main_page.rs` | Install, play the installed copy (or copy an update from its disc), uninstall. Items that do not apply cannot be chosen. |
| Language | `language_page.rs` | Choose the game's language. Not shown if the CD has only one, or if this build is installed already. The choice is loaded at once, so the following pages are in that language. |
| Drive | `drive_page.rs` | Windows only: choose the drive. On systems without drive letters the page is not part of the flow. |
| Folder | `folder_page.rs` | Type or accept the folder. A bad name shows why (`check_folder_name.rs`). |
| Not enough room | `room_page.rs` | Shown if the disk is too small; the player may go on anyway (`ignore_room.rs`). |
| Removal | `removal_page.rs` | For an uninstall: remove everything, keep the saves and settings, or go back. |

Between pages the installer does work that shows no page of its own: copying the files
(`install_files.rs`, with the progress display of `install_frames.rs`; Escape cancels),
removing the installation (`uninstall.rs`), making the installed copy the one that runs
(`open_installed.rs`), copying an update from its disc (`install_clio.rs`).

### The flow

The order of all this is ordinary code. `rc2k_core/src/install/flow.rs` names the pages
(`Page`) and the steps (`Step`: a page shown, or a piece of work) and says where each answer
leads (`after_page`, `after_copy`); `src/install/run_pages.rs` runs the steps one after another
until one leads nowhere.

| Step | Answer | Next |
| --- | --- | --- |
| Language (the first step of an installation) | a language | Main |
| | back | the installer ends |
| Main | install | Drive (Windows), Folder (elsewhere) |
| | play / update | the installed copy opened, or the update copied; the installer ends |
| | uninstall | Removal |
| | back | the installer ends |
| Drive | a drive | Folder |
| | back | Main |
| Folder | Enter | the copy |
| | Escape | Drive (Windows), Main (elsewhere) |
| The copy | done, failed or cancelled | the installer ends |
| | the folder cannot be used | Folder, which says why |
| | too little room | Not enough room |
| Not enough room | go on anyway | the copy |
| | back | Drive (Windows), Folder (elsewhere) |
| Removal (the first step with `-u`) | remove all, or keep the saves | the uninstall; the installer ends |
| | back | Main, or the installer ends when it began here |

The original also had a page for the kind of installation (how much to copy to the hard disk
and how much to leave on the CD), a DirectX set-up page and a registration page. None of them
exists here: the installation is always the full one. The game itself still reads the kind
from an installation's record, so an installation the 1999 installer made with files left on
the CD is still understood ([Install records](Install-records)).

## Copying the files

The action behind "install" is `install::install_files` (`src/install/install_files.rs`):

1. **Make the list.** The game has a built-in **install script**: a short byte program that
   says which folders to make and which files or whole folder trees to copy
   (`src/install/run_script.rs`). Running it against the source folder fills two tables, the
   folders and the files (`list_files.rs`, `list_folder.rs`, `list_folder_tree.rs`), and adds
   up the room needed.
2. **Check the destination.** The folder's name must be acceptable for the system and the disk
   must have room (`check_folder_name.rs`, `src/install/folders/`).
3. **Make the folders** (`make_folders.rs`).
4. **Copy**, one file at a time (`copy_counted.rs`), drawing a frame after each so the progress
   display moves and Escape is noticed.
5. **Write the record** and copy the readme for the chosen language.
6. **Install the program itself**: the running program and the libraries beside it are copied
   into the new folder, and menu entries are made (`src/install/shortcuts.rs`).

After the program has closed, the installed copy is started.

## The installation record

The original recorded an installation in the Windows registry. This program writes a small
file instead, `var/install.cfg` inside the installed folder (`src/install/portable.rs`). The
folder is therefore self-contained: it can be moved, copied to another machine, or put on a
handheld, and is still a valid installation. On reading, "where the game is installed" is
always the folder the program is running in.

The original's own record file in `var` (written by `src/install/write_record.rs`, read by
`read_record.rs`) is still written and read, so an installation made by the 1999 installer is
recognised too. Both files are described in [Install records](Install-records).

## Folders on each system

How a folder is named, checked, created and measured differs by system. That is behind one
trait, `Folders`, with an implementation per system chosen at build time
(`src/install/folders/mod.rs`):

| System | File | Default folder | Notes |
| --- | --- | --- | --- |
| Windows | `folders/windows.rs` | Chosen on the drive and folder pages | Drive letters, the drive page, Windows' forbidden characters. |
| Linux | `folders/unix.rs` | `$XDG_DATA_HOME/rally-championship`, usually `~/.local/share/rally-championship` | No drive page; `~` may be typed for the home folder. |
| macOS | `folders/unix.rs` | `~/Library/Application Support/Rally Championship` | The Unix rules with its own default. |

The game's file lists name files the Windows way, with backslashes and without regard to
case. On other systems `src/files/` translates each path as it is used, so the installer's own
code is the same everywhere.

Menu entries are also per system (`src/install/shortcuts.rs`): shortcut files in the Start
menu on Windows, desktop entries on Linux, small application bundles on macOS. There are three:
"Rally", "Uninstall Rally" and "View Readme". On Linux the game's entry has the game's icon,
which the installer writes into the user's icon theme
(`~/.local/share/icons/hicolor/512x512/apps/rally-championship.png`, removed with the entries),
and names the window class the program gives its window, so the desktop shows the running game
under that entry and its icon.

## Uninstalling

"Uninstall Rally" starts the installed program with `-u`. The main loop enters the installer,
which shows the removal page. The player chooses to remove everything, to keep the saved
games, or to cancel.

`install::uninstall` (`src/install/uninstall.rs`) then:

1. marks the record as "being removed";
2. runs the install script again to rebuild the lists of what was installed;
3. deletes each listed file, then each folder, innermost first;
4. with "remove everything", removes the installation's folder whole, including files the game
   made since (options, replays, logs);
5. clears the record and removes the menu entries.

A folder is only ever removed whole if it holds an installation record and is neither a drive's
root nor the home folder. On Windows, files still in use (the running program itself) are
removed after the program ends.

## Differences from the original

| Original | Here |
| --- | --- |
| Installs from the CD drive | Installs from the folder the program was started in. |
| Records the installation in the registry | `var/install.cfg` in the installed folder. |
| Asks for the kind of installation | Always the full installation. |
| Offers DirectX set-up and registration | Neither. |
| Adds Start menu items through the Windows shell's old program-group interface | Ordinary shortcuts (or the system's equivalent). |
| "Uninstall" runs the program from the CD | Runs the installed program with `-u`. |
| Registers the game for network lobbies | Not done; network play uses its own protocol ([Network play](Network-play)). |
| Windows only | Windows, Linux, macOS. The handheld has no installer step: an installed folder is copied to it. |

## Testing the installer

`tools/instest.sh` (Linux) and `tools/winstest.sh` (Windows under Wine) run the installer from
a throwaway copy of the CD's files into a throwaway home folder, accept every page, and check
that every file arrived and the record was written. The copy of the CD's files is named in the
environment variable `RC2K_DOWNLOAD`. See [Testing](Testing).

## Where to look

| To change or understand | Start in |
| --- | --- |
| When the installer runs | `src/system/main_loop.rs`, `src/install/portable.rs` |
| The installer's outline | `src/install/run_installer.rs`, `run_pages.rs`, `rc2k_core/src/install/flow.rs` |
| How a page works | `src/install/run_page.rs`, `page_key.rs`, `src/memory/list_handle.rs`, `src/install/list_pages.rs` |
| The pages' lists, pieces and links | `src/install/vars.rs`, `page_records.rs`, `page_links.rs` |
| What gets copied | `src/install/run_script.rs`, `install_files.rs` |
| Per-system folder rules | `src/install/folders/` |
| Menu entries and copying the program | `src/install/shortcuts.rs` |
| Removal | `src/install/uninstall.rs` |
| What happens after the program closes | `src/install/run_after_exit.rs` |

Next: [Network internals](Network-internals). See also [Module-install](Module-install),
[Install records](Install-records), [Differences from the original](Differences-from-the-original).
