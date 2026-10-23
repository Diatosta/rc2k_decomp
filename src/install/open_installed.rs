//! `install::open_installed` (0x413629): the installed folder (`g_installSource` normalised
//! into `byte_95E7F8`) made the current directory and `g_installing` set; if either the
//! normalise or the change fails, nothing is set.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD48` for the update script). Caller `install::run_page`. Its two
//! calls through `part`. Shadow mode runs the original alone (it changes directory).
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    INSTALLED, INSTALLING, NORMALISE, Part, SET_CD, SOURCE, a, at, both, p, quiet, region,
};
use crate::race::viewer::{Saved, dispatch_parts, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::open_installed", rva(0x413629), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SOURCE_PATH: usize = 0x51BB8D; // g_installSource

pub fn open_installed(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs {
        esi: a(SOURCE_PATH),
        edx: a(INSTALLED),
        ..*entry
    };
    part(at(NORMALISE), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    r.edx = a(INSTALLED);
    part(at(SET_CD), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    p(INSTALLING, 1);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        open_installed(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        open_installed(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea esi, g_installSource
    let copy = match crate::install::installer::copy(
        0x413629,
        0x27,
        &[0x60, 0x8D, 0x35, 0x8D, 0xBB, 0x51, 0x00],
        &[(0xD, 0x4139EC), (0x14, 0x403105)],
        &[0x4139EC, 0x403105],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let regions = [region(INSTALLING, 4)];
    let saved = Saved::new(&regions);
    let _ = (SOURCE, INSTALLED);
    for (norm_fail, cd_fail) in [(false, false), (true, false), (false, true)] {
        let prepare = || {
            p(INSTALLING, 0);
            crate::install::installer::answer(Box::new(move |place, r| {
                if place == a(0x4139EC) {
                    r.set_carry(norm_fail);
                } else if place == a(0x403105) {
                    r.set_carry(cd_fail);
                }
            }));
        };
        both(
            &mut t,
            &format!("normalise fails {norm_fail}, cd fails {cd_fail}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| open_installed(r, part),
        );
    }
    drop(saved);
    t.finish()
}
