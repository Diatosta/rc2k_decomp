//! `sound::stop_cd_audio` (0x40FCAA): `dword_518110` (the track playing) 0 and the CD
//! audio stopped five times: three calls of `sound::stop_cd_track`, then `call $+5` into
//! it, whose return lands on it once more, and from there back to the caller.
//!
//! In:  nothing
//! Out: every register preserved (`stop_cd_track` keeps them all)
//! Six callers: the CD audio's functions (0x40F8BB .. 0x40FDB9) and `system::main_loop`
//! (0x415439). Shadow mode runs the original alone. Self-test: a copy whose variable is the
//! test's, whose calls go to a stand-in and whose fall-through into `stop_cd_track` is
//! made a jump to it, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stop_cd_audio", rva(0x40FCAA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TRACK_RVA: usize = rva(0x518110); // dword_518110
const STOPS: u32 = 5;

/// A call: 0 `sound::stop_cd_track`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

// ---- Rust version ----

pub unsafe fn stop_cd_audio(entry: &Regs, track: u32, part: Part) -> Result<(), Refused> {
    unsafe { (track as *mut u32).write_unaligned(0) };
    let mut r = *entry;
    for _ in 0..STOPS {
        part(0, &mut r)?;
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let track = game_addr(TRACK_RVA) as u32;
    if let Err(e) =
        unsafe { stop_cd_audio(&entry, track, &mut crate::sound::stop_cd_track::game_part) }
    {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn note(place: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push((place, r.eax));
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    act(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut track = vec![0u32; 1];
    let at = track.as_mut_ptr() as u32;
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40FCAA,
            len: 0x23,
            entry: &[0xC7, 0x05, 0x10, 0x81, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00],
            calls: &[0x0A, 0x0F, 0x14, 0x19],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x40FCC8)).then(|| stand_in(0)),
        &[(0x518110, at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    unsafe {
        // +0x1E is `stop_cd_track` itself in the original: a jump to the stand-in there.
        c.patch_u32(0x1E, 0xE9);
        c.retarget(0x1F, stand_in(0));
    }
    set_handler(note);
    for eax in [0u32, 0x1234] {
        let run = |rust: bool| unsafe {
            (at as *mut u32).write_volatile(0xAAAA_AAAA);
            NOTED.lock().unwrap().clear();
            let before = Regs {
                eax,
                ..Regs::SENTINEL
            };
            let (done, preserved) = if rust {
                (stop_cd_audio(&before, at, &mut test_part).is_ok(), true)
            } else {
                let mut ro = before;
                call_game(&mut ro, c.addr());
                (true, ro.preserved_except(&before, 0))
            };
            (
                done && preserved,
                (at as *const u32).read_volatile(),
                std::mem::take(&mut *NOTED.lock().unwrap()),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs == mine && theirs.2.len() == STOPS as usize, || {
            format!("eax {eax:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(track);
    t.finish()
}
