//! `menu::list_take_key` (0x41469A): a list menu's key: the pending key taken
//! (`input::take_key`, eax 0 first); none: a millisecond's sleep (`system::sleep`). A key:
//! the repeat count `dword_51D40F` reset unless it is the last key again (`byte_51D413`)
//! and `dword_51D41F - (dword_51D423 + 6)` is negative (`js`), then one more; the key
//! into `byte_51D413`.
//!
//! In:  nothing
//! Out: eax the key (0 for none); with the last key again ebx `dword_51D41F` and ecx
//! `dword_51D423 + 6`; the flags of `test al, al` (no caller reads them)
//!
//! Callers: `race::replay_viewer`, `sub_4144CF`, `sub_4168CF`, `sub_4174C5`, `sub_417726`,
//! `sub_417A0D`.

use crate::install::vars::INSTALLER_PAGES;

/// What `list_take_key` gives: the key (eax, 0 for none) and, for the last key again, the
/// two clocks it compared (ebx `dword_51D41F`, ecx `dword_51D423 + 6`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Taken {
    pub key: u8,
    pub again: Option<(u32, u32)>,
}

impl Taken {}

impl Taken {
    /// The same in a flow's registers (`menu::event_flow::Flow`).
    pub fn carry_on(&self, f: &mut crate::menu::event_flow::Flow) {
        f.c.arg = u32::from(self.key);
        if let Some((frame, moved)) = self.again {
            f.c.palette = frame;
            f.c.measure_x = moved;
        }
        f.c.carry = false;
        f.zero = self.key == 0;
    }
}

/// The Rust version (see the header).
pub fn list_take_key() -> Taken {
    let key = crate::input::take_key::take_key();
    let mut again = None;
    if key == 0 {
        crate::system::sleep::sleep(1);
    } else {
        // Converted: `rc2k_core::menu::list_menu::repeats`.
        let repeated = key == INSTALLER_PAGES.last_key.get();
        let (mut frame, mut moved) = (0, 0);
        if repeated {
            (frame, moved) = (
                INSTALLER_PAGES.frame_clock.get(),
                INSTALLER_PAGES.move_clock.get(),
            );
            again = Some((frame, moved.wrapping_add(6)));
        }
        INSTALLER_PAGES
            .repeats
            .set(rc2k_core::menu::list_menu::repeats(
                INSTALLER_PAGES.repeats.get(),
                repeated,
                frame,
                moved,
            ));
        INSTALLER_PAGES.last_key.set(key);
    }
    Taken { key, again }
}
