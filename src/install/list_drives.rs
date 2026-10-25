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

use crate::install::installer::{DRIVE_COUNT, DRIVES, FIRST_DRIVE, a, g, game_api, p, wb, wd};
use rc2k_core::install::drives::{self, DriveMenu};

const MENU_8: usize = 0x51D7F8; // off_51D7F8
const MENU_16: usize = 0x51D8B0; // off_51D8B0
const MENU_MORE: usize = 0x51D9E8; // off_51D9E8
const CHOSEN_MENU: usize = 0x51D5F8; // off_51D5F8

pub fn list_drives() {
    let drives = a(DRIVES);
    let mut letter = g(FIRST_DRIVE);
    loop {
        // Converted: `rc2k_core::install::drives` (`offered`, `LISTED`, `free`, `text`,
        // `menu`, `items`).
        let kind = unsafe { crate::files::drive_type::drive_type(letter as u8) };
        if drives::offered(kind)
            && let Some(free) =
                crate::files::drive_free_space_kb::drive_free_space_kb(letter, &mut game_api)
        {
            let slot = drives + 8 * letter.wrapping_sub(b'a' as u32);
            wd(slot, g_at(slot).wrapping_add(1));
            wd(slot + 4, free);
        }
        letter = letter.wrapping_add(1);
        if letter > b'z' as u32 {
            break;
        }
    }
    p(DRIVE_COUNT, 0);
    let mut ebx = crate::install::store::DRIVE_ROWS.drive_rows.addr() as u32;
    // 'a' .. 'y': the loop's `cmp ecx, 7Ah; jb` leaves 'z' out (the game's slip, section 16)
    for letter in drives::LISTED {
        let slot = drives + 8 * (letter - b'a') as u32;
        if g_at(slot) == 0 {
            continue;
        }
        p(DRIVE_COUNT, g(DRIVE_COUNT) + 1);
        let row = ebx;
        wb(row, letter);
        let free = drives::free(g_at(slot + 4));
        let mut unit = Vec::new();
        unsafe {
            crate::strings::append_string_by_id::append_string_by_id(free.unit, &mut |from, n| {
                unit.extend_from_slice(std::slice::from_raw_parts(from, n));
            });
        }
        // the original writes the letter, skips a byte (`add ebx, 2`), then the rest
        for (n, &b) in drives::text(free, &unit).iter().enumerate() {
            wb(row + 2 + n as u32, b);
        }
        ebx = row + 0x10;
    }
    let count = g(DRIVE_COUNT);
    let menu = match drives::menu(count) {
        DriveMenu::Eight => MENU_8,
        DriveMenu::Sixteen => MENU_16,
        DriveMenu::More => MENU_MORE,
    };
    p(CHOSEN_MENU, a(menu));
    wb(a(menu) + 1, drives::items(count));
}

fn g_at(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}
