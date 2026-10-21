//! `race::viewer_snapshot` (0x41EFAB): the viewer's frame saved as a picture: the three
//! digits of `savedata\snapshot\000.pcx` one more (999 round to 000; the number found by
//! `race::find_snapshot_number`), the name as `dword_4F49C8` while the surface is locked
//! (`gfx::lock_surface`; failing, nothing), written (`gfx::write_screen_pcx`) and
//! unlocked (`gfx::unlock_surface`); `dword_4F49C8` put back.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read (the caller `test`s eax)
//!
//! Caller: `race::viewer_frame` (when `dword_5C2540` asks). Its calls through `part`: 1 the
//! lock (carry), 2 the writer, 3 the unlock; in the game as `gfx::take_screenshot`'s. Shadow
//! mode runs the original alone (it writes a file). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: digits to carry, the lock failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::write_screen_pcx::PICTURE_NAME;
use crate::physics::flow::a;
use crate::race::find_snapshot_number::{SNAPSHOT_DIGITS, SNAPSHOT_PATH};
use crate::race::viewer::{
    Part, Saved, answer, dispatch_parts, dword, get, put, region, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::viewer_snapshot", rva(0x41EFAB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves (the entry's).
pub fn viewer_snapshot(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let kept = get(PICTURE_NAME);
    let digits = a(SNAPSHOT_DIGITS) as *mut u8;
    let mut r = Regs {
        eax: a(SNAPSHOT_PATH),
        ..*entry
    };
    // Converted: `rc2k_core::race::viewer::next_snapshot`.
    unsafe {
        let (next, last, ecx) =
            rc2k_core::race::viewer::next_snapshot(digits.cast::<[u8; 3]>().read());
        digits.cast::<[u8; 3]>().write(next);
        r.ebx = digits.add(last) as u32;
        r.ecx = ecx;
    }
    put(PICTURE_NAME, r.eax);
    part(1, &mut r)?;
    if !r.carry() {
        part(2, &mut r)?;
        part(3, &mut r)?;
    }
    put(PICTURE_NAME, kept);
    Ok(*entry)
}

/// In the game: as `gfx::take_screenshot`'s.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::gfx::take_screenshot::game_part(place, r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        viewer_snapshot(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push dword_4F49C8
    let copy = match crate::race::viewer::copy(
        0x41EFAB,
        0x41,
        &[0x60, 0xFF, 0x35, 0xC8, 0x49, 0x4F, 0x00],
        &[0x28, 0x2F, 0x34],
        &[
            (crate::gfx::take_screenshot::LOCK, 1),
            (0x411E0C, 2),
            (crate::gfx::take_screenshot::UNLOCK, 3),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [region(SNAPSHOT_DIGITS, 3), dword(PICTURE_NAME)];
    let saved = Saved::new(&regions);
    for (digits, fails) in [
        (*b"000", false),
        (*b"019", false),
        (*b"999", false),
        (*b"099", true),
    ] {
        run_both(
            &mut t,
            &format!(
                "digits {:?}, the lock failing {fails}",
                std::str::from_utf8(&digits)
            ),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &|| {
                unsafe {
                    std::ptr::copy_nonoverlapping(digits.as_ptr(), a(SNAPSHOT_DIGITS) as *mut u8, 3)
                };
                put(PICTURE_NAME, 0x1357);
                answer(Box::new(move |place, r| {
                    if place == 1 {
                        r.set_carry(fails);
                    }
                }));
            },
            &|r| viewer_snapshot(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
