//! The shadow comparison of `audio::mix_channels` and `audio::write_mix`: what they write
//! (the regions given) after the Rust version and after the original, from the same
//! start; the original's result kept. Registers are not compared: their one caller,
//! `audio::mix`, keeps its own around them (`pusha`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, shadow};
use crate::screens::harness::CallOriginal;
use crate::service::harness::{Region, capture, restore};

pub fn dispatch_written(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    rust: &dyn Fn(&mut Regs) -> Result<(), Refused>,
) {
    hook.count_call();
    if !shadow() {
        if let Err(e) = rust(r) {
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
        }
        return;
    }
    if !crate::service::harness::shadow_due(hook) {
        unsafe { call_original(r) };
        return;
    }
    let before = capture(regions);
    let mut mine_regs = *r;
    let mine = rust(&mut mine_regs);
    let after_mine = capture(regions);
    restore(regions, &before);
    unsafe { call_original(r) };
    let theirs = capture(regions);
    match mine {
        Ok(()) if after_mine == theirs => {}
        Ok(()) => {
            let at = (0..theirs.len()).find(|&i| theirs[i] != after_mine[i]);
            hook.mismatch(|| format!("first byte differing at {at:x?} of {:#x}", theirs.len()));
        }
        Err(e) => hook.mismatch(|| format!("rust refused: {e:?}")),
    }
}
