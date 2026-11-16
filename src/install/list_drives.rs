//! The drives that can be installed to found and their menu built: each letter from `first_drive`
//! to 'z' that is not a CD-ROM or a RAM disk and whose free space can be read is recorded, then a
//! text (letter, space, free space as KB, MB or GB) is built in `drive_rows` for 'a' to 'y' (drive
//! Z is found but never listed, a slip kept as is; at most 24 drives are listed), and the drive
//! menu list that fits the count is given its number of items.

use crate::install::installer::wb;
use rc2k_core::install::drives;

pub fn list_drives(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) {
    // Per letter from 'a', a pair of dwords: how often it was found and its free space.
    let drives = &install.run.drives;
    let mut letter = u32::from(install.flags.first_drive.get());
    loop {
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
    let mut next_row = crate::memory::place::BytePlace::from(&install.drive_rows.drive_rows);
    // 'a' .. 'y': the loop's bound leaves 'z' out (the game's slip, section 16)
    let found: Vec<u8> = drives::LISTED
        .filter(|letter| drives[2 * (letter - b'a') as usize].get() != 0)
        .collect();
    // At most what the largest menu holds: with 25 drives the last, y, is not listed.
    for &letter in drives::kept(&found) {
        let slot = 2 * (letter - b'a') as usize;
        install
            .run
            .drive_count
            .set(install.run.drive_count.get() + 1);
        let row = next_row;
        wb(row, letter);
        let free = drives::free(drives[slot + 1].get());
        let mut unit = Vec::new();
        crate::menu::event_flow::refuse_here(
            crate::strings::append_string_by_id::append_string_by_id(
                crate::strings::append_string_by_id::StringOwners::of(o),
                free.unit,
                &mut |bytes| {
                    unit.extend_from_slice(bytes);
                },
            ),
            "install::list_drives: a string",
        );
        // the letter, a skipped byte, then the rest
        for (n, &b) in drives::text(free, &unit).iter().enumerate() {
            wb(row.add(2 + n as u32), b);
        }
        next_row = row.add(0x10);
    }
    let count = install.run.drive_count.get();
    let list = drive_list(count);
    crate::memory::list_handle::ListMenu::installer(install, list).set_count(drives::items(count));
}

/// The drive page's list for `count` drives.
pub fn drive_list(count: u32) -> crate::install::page_links::ListId {
    use crate::install::page_links::ListId;
    match drives::menu(count) {
        drives::DriveMenu::Eight => ListId::Drives8,
        drives::DriveMenu::Sixteen => ListId::Drives16,
        drives::DriveMenu::More => ListId::DrivesMore,
    }
}
