//! The drives the game may be installed to (converted 2026-10-21 from
//! `install::list_drives` 0x413C27): which are taken, each one's text on the drive page
//! ("c 512 MB": the letter, a byte not written, a space, the free space) and the page's
//! menu by how many there are.

use crate::strings::format::{Pad, decimal};
use std::ops::RangeInclusive;

/// A CD-ROM's drive type and a RAM disk's: not offered.
pub const CD_ROM: u32 = 5;
pub const RAM_DISK: u32 = 6;

/// String ids of the units.
pub const KB: u32 = 0xA6;
pub const MB: u32 = 0xA7;
pub const GB: u32 = 0xA8;

/// Whether a drive of `drive_type` is looked at further (its free space asked).
pub fn offered(drive_type: u32) -> bool {
    drive_type != CD_ROM && drive_type != RAM_DISK
}

/// The letters listed on the page: 'a' to 'y' (`cmp ecx, 7Ah; jb`: a drive Z found is never
/// listed, the game's slip).
pub const LISTED: RangeInclusive<u8> = b'a'..=b'y';

/// Free space as the page shows it: the number, its unit's string id, and whether ">="
/// goes before it (in GB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Free {
    pub number: u32,
    pub unit: u32,
    pub at_least: bool,
}

/// `kb` free KB in the largest unit it reaches (shifts of 10 bits).
pub fn free(kb: u32) -> Free {
    if kb < 0x400 {
        return Free {
            number: kb,
            unit: KB,
            at_least: false,
        };
    }
    let mb = kb >> 10;
    if mb < 0x400 {
        return Free {
            number: mb,
            unit: MB,
            at_least: false,
        };
    }
    Free {
        number: mb >> 10,
        unit: GB,
        at_least: true,
    }
}

/// A drive's text from its third byte on: a space, ">=" for GB, the number, the unit's
/// text, a NUL.
pub fn text(free: Free, unit: &[u8]) -> Vec<u8> {
    let mut out = vec![b' '];
    if free.at_least {
        out.extend(b">=");
    }
    out.extend(decimal(free.number, Pad::None));
    out.extend(unit);
    out.push(0);
    out
}

/// The drive page's menu by the drives listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveMenu {
    /// 8 or fewer (`off_51D7F8`).
    Eight,
    /// 16 or fewer (`off_51D8B0`).
    Sixteen,
    /// More (`off_51D9E8`).
    More,
}

pub fn menu(count: u32) -> DriveMenu {
    if count <= 8 {
        DriveMenu::Eight
    } else if count <= 16 {
        DriveMenu::Sixteen
    } else {
        DriveMenu::More
    }
}

/// The menu's item count: the drives and two more.
pub fn items(count: u32) -> u8 {
    (count + 2) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drives() {
        assert!(offered(3));
        assert!(!offered(CD_ROM));
        assert!(!LISTED.contains(&b'z'));
        assert_eq!(
            free(0x3FF),
            Free {
                number: 0x3FF,
                unit: KB,
                at_least: false
            }
        );
        assert_eq!(
            free(0x400),
            Free {
                number: 1,
                unit: MB,
                at_least: false
            }
        );
        assert_eq!(
            free(0x10_0000),
            Free {
                number: 1,
                unit: GB,
                at_least: true
            }
        );
        assert_eq!(text(free(0x800), b"MB"), b" 2MB\0".to_vec());
        assert_eq!(text(free(0x30_0000), b"GB"), b" >=3GB\0".to_vec());
        assert_eq!(menu(8), DriveMenu::Eight);
        assert_eq!(menu(9), DriveMenu::Sixteen);
        assert_eq!(menu(17), DriveMenu::More);
        assert_eq!(items(5), 7);
    }
}
