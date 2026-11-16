//! The setup slots' list (three pages) and the name being typed; the keys of the setup's edit
//! pages and their numbers; the highlighted slot; a loaded setup and the car's values the edit
//! pages start from.

use crate::menu::player::Player;
use crate::saves::kept::SETUP_LEN;

/// The slot list's next page (0 after 2).
pub fn next_page(page: i32) -> i32 {
    let next = page.wrapping_add(1);
    if next < 3 { next } else { 0 }
}

/// The last character taken off a name of `len` characters (its byte made 0); the new
/// length. At 0 nothing changes.
pub fn backspace(name: &mut [u8], len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    name[len - 1] = 0;
    len - 1
}

/// What a key does on an edit page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageKey {
    /// No key: nothing, not even the clock noted.
    None,
    /// Down arrow (0x68): one step down.
    Down,
    /// Up arrow (0x69): one step up.
    Up,
    /// Return (0x1C): the value taken.
    Confirm,
    /// Any other key: only the clock noted.
    Other,
}

pub fn page_key(key: u8) -> PageKey {
    match key {
        0 => PageKey::None,
        0x68 => PageKey::Down,
        0x69 => PageKey::Up,
        0x1C => PageKey::Confirm,
        _ => PageKey::Other,
    }
}

/// The slot lists' pages hold this many slots.
pub const SLOTS_PER_PAGE: u32 = 30;

/// The slot the highlighted item of a slot list stands for: page x 30 + item - 1, one less
/// from item 0x11 on (signed; past the list's 99 slots the caller refuses).
pub fn highlighted_slot(page: u32, item: u32) -> u32 {
    let slot = page
        .wrapping_mul(SLOTS_PER_PAGE)
        .wrapping_add(item)
        .wrapping_sub(1);
    if (item as i32) >= 0x11 {
        slot.wrapping_sub(1)
    } else {
        slot
    }
}

/// The service area's edit pages, by their number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditPage {
    GearRatio = 1,
    Tyre,
    SteeringSensitivity,
    BrakeBalance,
    RideHeight,
    SuspensionStiffness,
}

impl EditPage {
    /// `None` for a page number that does not exist (fatal error 0x80709A9A, "Incorrect Locked
    /// Box", for the caller).
    pub fn from_number(n: u32) -> Option<Self> {
        Some(match n {
            1 => EditPage::GearRatio,
            2 => EditPage::Tyre,
            3 => EditPage::SteeringSensitivity,
            4 => EditPage::BrakeBalance,
            5 => EditPage::RideHeight,
            6 => EditPage::SuspensionStiffness,
            _ => return None,
        })
    }
}

/// The order a loaded setup's changes are confirmed in (each only if its chosen value
/// differs from the installed one).
pub const CONFIRM_ORDER: [EditPage; 4] = [
    EditPage::GearRatio,
    EditPage::Tyre,
    EditPage::RideHeight,
    EditPage::SuspensionStiffness,
];

/// A setup file's six values (+0x30 ..): the chosen tyre, the car's brake balance, the
/// chosen ride height and stiffness, the car's steering sensitivity, the chosen gearing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetupValues {
    pub tyre: u32,
    pub brake_balance: u32,
    pub ride_height: u32,
    pub suspension_stiffness: u32,
    pub steering_sensitivity: u32,
    pub gear_ratio: u32,
}

impl SetupValues {
    pub fn from_file(file: &[u8; SETUP_LEN]) -> Self {
        let v = |n: usize| u32::from_le_bytes(file[0x30 + 4 * n..0x34 + 4 * n].try_into().unwrap());
        SetupValues {
            tyre: v(0),
            brake_balance: v(1),
            ride_height: v(2),
            suspension_stiffness: v(3),
            steering_sensitivity: v(4),
            gear_ratio: v(5),
        }
    }
}

impl Player {
    /// What `load_setup_values` copies into the edit pages' pairs: the tyre (+0x10C), the
    /// gearing (+0x120), the suspension's stiffness (+0x118).
    pub fn setup_edit_values(&self) -> [u32; 3] {
        [self.tyre, self.gear_ratio, self.suspension_stiffness]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setups() {
        assert_eq!((next_page(2), next_page(0), next_page(-5)), (0, 1, -4));
        let mut n = *b"abc\0";
        assert_eq!((backspace(&mut n, 3), n), (2, *b"ab\0\0"));
        assert_eq!(backspace(&mut n, 0), 0);
    }

    #[test]
    fn page_keys() {
        assert_eq!(page_key(0), PageKey::None);
        assert_eq!(page_key(0x68), PageKey::Down);
        assert_eq!(page_key(0x69), PageKey::Up);
        assert_eq!(page_key(0x1C), PageKey::Confirm);
        assert_eq!(page_key(0x1D), PageKey::Other);
    }

    #[test]
    fn slots_and_pages() {
        assert_eq!(highlighted_slot(0, 1), 0);
        assert_eq!(highlighted_slot(1, 0x10), 30 + 0xF);
        assert_eq!(highlighted_slot(1, 0x11), 30 + 0xF);
        assert_eq!(highlighted_slot(0, 0), u32::MAX);
        assert_eq!(EditPage::from_number(4), Some(EditPage::BrakeBalance));
        assert_eq!(EditPage::from_number(7), None);
        let mut f = [0u8; SETUP_LEN];
        for n in 0..6u32 {
            f[0x30 + 4 * n as usize] = n as u8 + 1;
        }
        let v = SetupValues::from_file(&f);
        assert_eq!((v.tyre, v.steering_sensitivity, v.gear_ratio), (1, 5, 6));
    }
}
