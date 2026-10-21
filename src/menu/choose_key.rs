//! What the twelve `menu::choose_key_0` .. `_11` (0x4682D3 .. 0x4683BA, 0x15 bytes apart)
//! share: the actions of the redefine keys pages' items, each starting the redefining of
//! its key's place (`menu/redefine_keys.rs`): `g_redefiningKey` = 1 and
//! `g_redefinedKeyPlace` = the place. `choose_key_8` is in no menu.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write};
use crate::menu::redefine_keys::{PLACE_RVA, REDEFINING_RVA};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};

pub fn choose_key(place: u32) {
    // Converted: `rc2k_core::menu::player::Redefining`.
    let r = rc2k_core::menu::player::Redefining::start(place);
    unsafe {
        game_write(REDEFINING_RVA, r.on);
        game_write(PLACE_RVA, r.place);
    }
}

pub fn dispatch_choose(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, place: u32) {
    let rust = move |_: &Regs| -> Result<Left, Refused> {
        choose_key(place);
        Ok(Left::nothing())
    };
    dispatch_state(
        hook,
        r,
        call_original,
        &[at(REDEFINING_RVA), at(PLACE_RVA)],
        &rust,
    );
}

pub fn check_choose(t: &mut SelfTest, call_original: CallOriginal, place: u32) {
    let regions = [at(REDEFINING_RVA), at(PLACE_RVA)];
    let saved = capture(&regions);
    let rust = move |_: &Regs| -> Result<Left, Refused> {
        choose_key(place);
        Ok(Left::nothing())
    };
    for (redefining, before) in [(0u32, 999u32), (1, 3), (0, place)] {
        check_state(
            t,
            &format!("redefining {redefining}, place {before}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(REDEFINING_RVA, redefining);
                game_write(PLACE_RVA, before);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
}
