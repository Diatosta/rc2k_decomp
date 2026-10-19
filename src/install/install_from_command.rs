//! `install::install_from_command` (0x412F34): the install driven by the `-i` command line
//! (two paths already in `byte_95E6F8`/`byte_95E7F8` and two digits in `dword_51BA08` /
//! `dword_51BA0C`). The game folder is made current (`files::set_current_directory`); base
//! directory B (`unk_5295B4`) and the installed folder are normalised
//! (`install::normalise_path`); the given media mode is clamped to 0..3 into `g_mediaMode`
//! (only when above 0), and only then a given language (above 0) chooses and loads it
//! (`install::choose_language`, `strings::load_language`); then the files are installed
//! (`install::install_files`). Any failing directory change or normalise ends it early.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. Its calls through `part`. Ungated. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (carries and the digits
//! scripted), against the Rust version; `g_mediaMode` and the call sequence compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    BASE_B, CHOOSE_LANGUAGE, GAME_DIR, GIVEN_LANGUAGE, GIVEN_MODE, INSTALLED, LOAD_LANGUAGE,
    MEDIA_MODE, NORMALISE, Part, RUN_SCRIPT, SET_CD, TYPED, a, at, both, g, p, quiet, region,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "install::install_from_command",
    rva(0x412F34),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const INSTALL_FILES: usize = 0x4130CD;

pub fn install_from_command(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = a(GAME_DIR);
    part(at(SET_CD), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    r.esi = a(BASE_B);
    r.edx = a(GAME_DIR);
    part(at(NORMALISE), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    r.esi = a(INSTALLED);
    r.edx = a(TYPED);
    part(at(NORMALISE), &mut r)?;
    // A given media mode (above 0) is taken, held to 3, and only then a given language
    // (above 0): a mode of 0 or less goes straight to the install. eax as the original has
    // it at each call.
    r.eax = g(GIVEN_MODE);
    if (r.eax as i32) > 0 {
        r.eax = (r.eax as i32).min(3) as u32;
        p(MEDIA_MODE, r.eax);
        r.eax = g(GIVEN_LANGUAGE);
        if (r.eax as i32) > 0 {
            part(at(CHOOSE_LANGUAGE), &mut r)?;
            part(at(LOAD_LANGUAGE), &mut r)?;
        }
    }
    part(at(INSTALL_FILES), &mut r)?;
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        install_from_command(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let _ = RUN_SCRIPT;
    // pusha; lea edx, byte_95E6F8
    let copy = match crate::install::installer::copy(
        0x412F34,
        0x6E,
        &[0x60, 0x8D, 0x15, 0xF8, 0xE6, 0x95, 0x00],
        &[
            (0x7, 0x403105),
            (0x1A, 0x4139EC),
            (0x2D, 0x4139EC),
            (0x5D, 0x41397A),
            (0x62, 0x40331F),
            (0x67, 0x4130CD),
        ],
        &[0x403105, 0x4139EC, 0x41397A, 0x40331F, 0x4130CD],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let regions = [
        region(MEDIA_MODE, 4),
        region(GIVEN_MODE, 4),
        region(GIVEN_LANGUAGE, 4),
    ];
    let saved = crate::race::viewer::Saved::new(&regions);
    for (cd_fail, norm1_fail, mode, lang) in [
        (false, false, 2u32, 1u32),
        (false, false, 0, 0),
        (false, false, 5, 0),
        (true, false, 2, 1),
        (false, true, 2, 1),
        (false, false, 0xFFFF_FFFF, 1),
    ] {
        let prepare = || {
            crate::install::installer::answer(Box::new(move |place, r| {
                if place == a(SET_CD) {
                    r.set_carry(cd_fail);
                } else if place == a(NORMALISE) {
                    // the first normalise (esi=base B) can fail; the second never does here
                    r.set_carry(norm1_fail && r.esi == a(BASE_B));
                }
            }));
            p(MEDIA_MODE, 7);
            p(GIVEN_MODE, mode);
            p(GIVEN_LANGUAGE, lang);
        };
        both(
            &mut t,
            &format!("cd_fail {cd_fail}, norm1_fail {norm1_fail}, mode {mode}, lang {lang}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| install_from_command(r, part),
        );
    }
    quiet();
    drop(saved);
    t.finish()
}
