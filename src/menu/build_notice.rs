//! `menu::build_notice` (0x4165F1): for the edition whose notice kind is 2, makes one
//! copyright line of the two.
//!
//! Only when `g_noticeKind` (the byte at 0x4E07A0, from the language's `.loc` file) is 2. The first
//! line (the first copyright line, "Copyright (c) Magnetic Fields (Software Design) Ltd, 1999. All rights
//! reserved.") becomes "(c) 2000 Actualize Ltd and Magnetic Fields (Software Design) Ltd, " and the
//! second (the second copyright line, "Copyright (c) Actualize Ltd, 1999. ..") is emptied. It is put together
//! in the shared text buffer from pieces of the two lines themselves: two characters from the
//! second line's eleventh on, "2000", fourteen from its twelfth on, " and", 0x26 from the first
//! line's twelfth on, a space and a NUL; then copied over the first line. One caller,
//! `system::main_loop`. Calling it twice builds from the changed lines, as the original would. Rust
//! callers run its `entry` (`routes.rs`).

// ---- game data ----

use crate::strings::owner::Strings;
use rc2k_core::menu::notice::{
    BUILDS, BUILT_LEN, FIRST_FROM, FIRST_READ, SECOND_FROM, SECOND_READ, notice,
};

pub fn build_notice(menus: &Menus, strings: &Strings) {
    let panels = &menus.panels;
    // The logic is in `rc2k_core::menu::notice::notice`.
    if crate::strings::load_language::notice_kind(strings) != BUILDS {
        return;
    }
    let first: [u8; FIRST_READ] =
        std::array::from_fn(|i| panels.notice.notice_first.get(FIRST_FROM + i));
    let second: [u8; SECOND_READ] =
        std::array::from_fn(|i| panels.notice.notice_second.get(SECOND_FROM + i));
    let built = notice(&first, &second);
    for (i, &byte) in built.iter().take(BUILT_LEN).enumerate() {
        strings.text.buffer.text.set(i, byte);
        panels.notice.notice_first.set(i, byte);
    }
    panels.notice.notice_second.set(0, 0);
}

// ---- the notice's lines (IDA 0x52A263 .. 0x52A30F, split from the network's block) ----

use crate::menu::owner::Menus;
use crate::session::{PlainBytes, with_runs};

/// IDA 0x52A263 .. 0x52A30F.
#[derive(rc2k_derive::Cells)]
pub struct BuildNotice {
    /// 0x52A263: `unk_52A263`: the build notice's first line, "Copyright ... Magnetic Fields ..." (`menu::build_notice`).
    pub notice_first: PlainBytes<78>,
    /// 0x52A2B1: `byte_52A2B1`: the build notice's second line, "Copyright ... Actualize Ltd ...", and the web address after it.
    pub notice_second: PlainBytes<94>,
}

impl BuildNotice {
    pub const fn new() -> Self {
        BuildNotice {
    notice_first: PlainBytes::new(with_runs::<78>(&[(0, b"Copyright \xa9 Magnetic Fields (Software Design) Ltd, 1999. All rights reserved.")])),
    notice_second: PlainBytes::new(with_runs::<94>(&[(0, b"Copyright \xa9 Actualize Ltd, 1999. All rights reserved.\x00http:\x01/\x01/\x01www.rallychampionship.co.uk\x01/")])),
        }
    }
}
