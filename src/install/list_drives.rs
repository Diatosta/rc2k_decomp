//! `install::list_drives` (0x413C27): the drives that can be installed to found and their
//! menu built. Each letter from `first_drive` ('c') to 'z' that is not a CD-ROM (5) or a
//! RAM disk (6) and whose free space can be read (`files::drive_free_space_kb`) is recorded
//! at `drives[letter-'a']` (count 1, free KB). Then for each recorded letter a text is
//! built in `drive_rows` (0x10 bytes each; 'a' to 'y': drive Z is found but never listed,
//! the game's slip): the letter, a space, the free space as KB
//! (id 0xA6), MB (0xA7) or GB (0xA8, with ">=" before it), NUL. The number is in
//! `drive_count`; the drive menu list is chosen (the 8-drive menu up to 8 drives, the
//! 16-drive menu up to 16, else the largest) into the drive page's entry with its item count set to the number
//! plus two.
//!
//! Caller `install::run_installer`. Its two probes: `files::drive_type`,
//! `files::drive_free_space_kb`.

use crate::install::installer::wb;
use crate::install::page_links::{ListId, PageLink};
use rc2k_core::install::drives::{self, DriveMenu};

/// The drive page's entry in the pages' table (`g_driveMenu`, 0x51D5F8, from 0x51D594).
const DRIVE_ENTRY: usize = 25;

pub fn list_drives(install: &'static crate::install::owner::Install, o: crate::owners::Owners<'_>) {
    // Per letter from 'a', a pair of dwords: how often it was found and its free space.
    let drives = &install.run.drives;
    let mut letter = install.run.first_drive.get();
    loop {
        // Converted: `rc2k_core::install::drives` (`offered`, `LISTED`, `free`, `text`,
        // `menu`, `items`).
        let kind = crate::files::drive_type::drive_type(letter as u8);
        if drives::offered(kind)
            && let Some(free) =
                crate::files::drive_free_space_kb::drive_free_space_kb(o.system, letter)
        {
            // The first letter is 'a' or 'c', the last 'z': inside the 26 pairs.
            let slot = 2 * (letter - b'a' as u32) as usize;
            drives[slot].set(drives[slot].get().wrapping_add(1));
            drives[slot + 1].set(free);
        }
        letter = letter.wrapping_add(1);
        if letter > b'z' as u32 {
            break;
        }
    }
    install.run.drive_count.set(0);
    let mut ebx = crate::place::BytePlace::from(&install.drive_rows.drive_rows);
    // 'a' .. 'y': the loop's bound leaves 'z' out (the game's slip, section 16)
    for letter in drives::LISTED {
        let slot = 2 * (letter - b'a') as usize;
        if drives[slot].get() == 0 {
            continue;
        }
        install
            .run
            .drive_count
            .set(install.run.drive_count.get() + 1);
        let row = ebx;
        wb(row, letter);
        let free = drives::free(drives[slot + 1].get());
        let mut unit = Vec::new();
        crate::menu::event_flow::refuse_here(
            crate::strings::append_string_by_id::append_string_by_id_in(
                o,
                free.unit,
                &mut |bytes| {
                    unit.extend_from_slice(bytes);
                },
            ),
            "install::list_drives: a string",
        );
        // the original writes the letter, skips a byte, then the rest
        for (n, &b) in drives::text(free, &unit).iter().enumerate() {
            wb(row.add(2 + n as u32), b);
        }
        ebx = row.add(0x10);
    }
    let count = install.run.drive_count.get();
    let list = match drives::menu(count) {
        DriveMenu::Eight => ListId::Drives8,
        DriveMenu::Sixteen => ListId::Drives16,
        DriveMenu::More => ListId::DrivesMore,
    };
    // The drive page's entry links to the menu (`page_links::PageLink`).
    install.pages.page_table[DRIVE_ENTRY].set(PageLink::list(list).raw());
    crate::menu::list_handle::ListMenu::installer(list).set_count(drives::items(count));
}
