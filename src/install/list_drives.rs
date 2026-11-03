//! `install::list_drives` (0x413C27): the drives that can be installed to found and their
//! menu built. Each letter from `dword_51BAE4` ('c') to 'z' that is not a CD-ROM (5) or a
//! RAM disk (6) and whose free space can be read (`files::drive_free_space_kb`) is recorded
//! at `dword_51BA10[letter-'a']` (count 1, free KB). Then for each recorded letter a text is
//! built in `byte_52A30F` (0x10 bytes each; 'a' to 'y': drive Z is found but never listed,
//! the game's slip): the letter, a space, the free space as KB
//! (id 0xA6), MB (0xA7) or GB (0xA8, with ">=" before it), NUL. The number is in
//! `dword_51BAE0`; the drive menu list is chosen (`off_51D7F8` up to 8 drives, `off_51D8B0`
//! up to 16, else `off_51D9E8`) into `off_51D5F8` with its item count set to the number
//! plus two.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. Its two probes: `files::drive_type`,
//! `files::drive_free_space_kb`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{wb, wd};
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_RUN;
use rc2k_core::install::drives::{self, DriveMenu};

pub fn list_drives() {
    let drives = crate::place::Place::from(crate::pointers::GamePtr::to(&INSTALLER_RUN.drives));
    let mut letter = INSTALLER_RUN.first_drive.get();
    loop {
        // Converted: `rc2k_core::install::drives` (`offered`, `LISTED`, `free`, `text`,
        // `menu`, `items`).
        let kind = crate::files::drive_type::drive_type(letter as u8);
        if drives::offered(kind)
            && let Some(free) = crate::files::drive_free_space_kb::drive_free_space_kb(letter)
        {
            let slot = drives.add(8u32.wrapping_mul(letter.wrapping_sub(b'a' as u32)));
            wd(slot, g_at(slot).wrapping_add(1));
            wd(slot.add(4), free);
        }
        letter = letter.wrapping_add(1);
        if letter > b'z' as u32 {
            break;
        }
    }
    INSTALLER_RUN.drive_count.set(0);
    let mut ebx = crate::place::Place::from(&crate::install::store::DRIVE_ROWS.drive_rows);
    // 'a' .. 'y': the loop's `cmp ecx, 7Ah; jb` leaves 'z' out (the game's slip, section 16)
    for letter in drives::LISTED {
        let slot = drives.add(8 * (letter - b'a') as u32);
        if g_at(slot) == 0 {
            continue;
        }
        INSTALLER_RUN
            .drive_count
            .set(INSTALLER_RUN.drive_count.get() + 1);
        let row = ebx;
        wb(row, letter);
        let free = drives::free(g_at(slot.add(4)));
        let mut unit = Vec::new();
        crate::menu::event_flow::refuse_here(
            crate::strings::append_string_by_id::append_string_by_id(free.unit, &mut |bytes| {
                unit.extend_from_slice(bytes);
            }),
            "install::list_drives: a string",
        );
        // the original writes the letter, skips a byte (`add ebx, 2`), then the rest
        for (n, &b) in drives::text(free, &unit).iter().enumerate() {
            wb(row.add(2 + n as u32), b);
        }
        ebx = row.add(0x10);
    }
    let count = INSTALLER_RUN.drive_count.get();
    let (menu, name) = match drives::menu(count) {
        DriveMenu::Eight => (crate::place::Place::from(&INSTALLER_PAGES.menu_8), 0x51D7F8),
        DriveMenu::Sixteen => (
            crate::place::Place::from(&INSTALLER_PAGES.menu_16),
            0x51D8B0,
        ),
        DriveMenu::More => (
            crate::place::Place::from(&INSTALLER_PAGES.menu_more),
            0x51D9E8,
        ),
    };
    // A link: the menu's IDA address, as the image's (`InstallerPages::resolved`).
    INSTALLER_PAGES.chosen_menu.set_u32(0, name);
    crate::menu::list_handle::ListMenu::of(menu).set_count(drives::items(count));
}

fn g_at(at: crate::place::Place) -> u32 {
    crate::install::installer::rd(at)
}
