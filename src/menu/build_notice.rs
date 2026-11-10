//! `menu::build_notice` (0x4165F1): for the edition whose notice kind is 2, makes one
//! copyright line of the two.
//!
//! Only when `g_noticeKind` (the byte at 0x4E07A0, from the language's `.loc` file) is 2. The first
//! line (`unk_52A263`, "Copyright (c) Magnetic Fields (Software Design) Ltd, 1999. All rights
//! reserved.") becomes "(c) 2000 Actualize Ltd and Magnetic Fields (Software Design) Ltd, " and the
//! second (`byte_52A2B1`, "Copyright (c) Actualize Ltd, 1999. ..") is emptied. It is put together
//! in the shared text buffer from pieces of the two lines themselves: two characters from the
//! second line's eleventh on, "2000", fourteen from its twelfth on, " and", 0x26 from the first
//! line's twelfth on, a space and a NUL; then copied over the first line. One caller,
//! `system::main_loop`. Calling it twice builds from the changed lines, as the original would. Rust
//! callers run its `entry` (`routes.rs`).

// ---- game data ----

use crate::strings::owner::strings;
use rc2k_core::menu::notice::{
    BUILDS, BUILT_LEN, FIRST_FROM, FIRST_READ, SECOND_FROM, SECOND_READ, notice,
};

pub fn build_notice(menus: &Menus) {
    let panels = &menus.panels;
    // Converted: `rc2k_core::menu::notice::notice`.
    if crate::strings::load_language::notice_kind() != BUILDS {
        return;
    }
    let first: [u8; FIRST_READ] =
        std::array::from_fn(|i| panels.notice.notice_first.get(FIRST_FROM + i));
    let second: [u8; SECOND_READ] =
        std::array::from_fn(|i| panels.notice.notice_second.get(SECOND_FROM + i));
    let built = notice(&first, &second);
    for (i, &byte) in built.iter().take(BUILT_LEN).enumerate() {
        strings().text.buffer.text.set(i, byte);
        panels.notice.notice_first.set(i, byte);
    }
    panels.notice.notice_second.set(0, 0);
}

// ---- the notice's lines (IDA 0x52A263 .. 0x52A30F, split from the network's block) ----

use crate::menu::owner::Menus;
use crate::session::{Bytes, with_runs};

/// IDA 0x52A263 .. 0x52A30F.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct BuildNotice {
    /// 0x52A263: `unk_52A263`: the build notice's first line, "Copyright ... Magnetic Fields ..." (`menu::build_notice`).
    pub notice_first: Bytes<78>,
    /// 0x52A2B1: `byte_52A2B1`: the build notice's second line, "Copyright ... Actualize Ltd ...", and the web address after it.
    pub notice_second: Bytes<94>,
}

impl BuildNotice {
    pub const fn new() -> Self {
        BuildNotice {
    notice_first: Bytes::new(with_runs::<78>(&[(0, b"Copyright \xa9 Magnetic Fields (Software Design) Ltd, 1999. All rights reserved.")])),
    notice_second: Bytes::new(with_runs::<94>(&[(0, b"Copyright \xa9 Actualize Ltd, 1999. All rights reserved.\x00http:\x01/\x01/\x01www.rallychampionship.co.uk\x01/")])),
        }
    }
}

impl BuildNotice {
    pub const START: usize = 0x52A263;
    pub const LEN: usize = 0xAC;
}

#[cfg(test)]
mod notice_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn notice_starts_as_the_games_image() {
        assert_eq!(size_of::<BuildNotice>(), BuildNotice::LEN);
        let p = (&raw const crate::menu::owner::MENUS.panels.notice).cast::<u8>();
        let image = crate::moved::image_bytes(BuildNotice::START, BuildNotice::LEN);
        for (i, &b) in image.iter().enumerate() {
            assert_eq!(
                unsafe { p.add(i).read() },
                b,
                "{:#x}",
                BuildNotice::START + i
            );
        }
        assert_eq!(
            offset_of!(BuildNotice, notice_second),
            0x52A2B1 - BuildNotice::START
        );
    }
}
