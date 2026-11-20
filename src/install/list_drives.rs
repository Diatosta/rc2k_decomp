//! The drives that can be installed to found and their menu built: each letter from `first_drive`
//! to 'z' that is not a CD-ROM or a RAM disk and whose free space can be read is recorded, then a
//! text (letter, space, free space as KB, MB or GB) is built in `drive_rows` for 'a' to 'y' (drive
//! Z is found but never listed, a slip kept as is; at most 24 drives are listed), and the drive
//! count is kept (the drive page's items are made from it).

use crate::install::vars::DriveFound;
use rc2k_core::install::drives;

pub(super) fn list_drives(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) {
    // Per letter from 'a', a pair of dwords: how often it was found and its free space.
    let drives = &install.run.drives;
    let found_at = |letter: u8| &drives[(letter - b'a') as usize];
    let mut letter = u32::from(install.flags.first_drive.get());
    loop {
        let kind = crate::files::drive_type::drive_type(letter as u8);
        if drives::offered(kind)
            && let Some(free) = crate::files::drive_free_space_kb::drive_free_space_kb(
                &o.system.work().files,
                letter,
            )
        {
            // The first letter is 'a' or 'c', the last 'z': inside the 26 drives.
            let drive = found_at(letter as u8);
            drive.set(DriveFound {
                found: drive.get().found.wrapping_add(1),
                free_kb: free,
            });
        }
        letter = letter.wrapping_add(1);
        if letter > b'z' as u32 {
            break;
        }
    }
    install.run.drive_count.set(0);
    let rows = &install.drive_rows.drive_rows;
    let mut next_row = 0usize;
    // 'a' .. 'y': the loop's bound leaves 'z' out (the game's slip, section 16)
    let found: Vec<u8> = drives::LISTED
        .filter(|&letter| found_at(letter).get().found != 0)
        .collect();
    // At most what the largest menu holds: with 25 drives the last, y, is not listed.
    for &letter in drives::kept(&found) {
        install
            .run
            .drive_count
            .set(install.run.drive_count.get() + 1);
        let row = next_row;
        rows.set(row, letter);
        let free = drives::free(found_at(letter).get().free_kb);
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
            rows.set(row + 2 + n, b);
        }
        next_row = row + 0x10;
    }
}
