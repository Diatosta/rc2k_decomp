//! `race::replay_screenshot` (0x41F3DB): while the replay's frames are being recorded
//! (`dword_5C2CCC` 1, a reading), the frame saved as a picture: the clock started at the
//! first (`dword_5C2CD4` 1, `dword_5C2CD0` = `GetTickCount`); the six digits of the name
//! `rpl\000000.pcx` (0x5C2CDC) one more (each past '9' back to '0' and the one before
//! raised; all six at most); the name as `dword_4F49C8` while the surface is locked
//! (`gfx::lock_surface`; carry: not saved), written (`sub_411E0C`, the PCX writer) and
//! unlocked (`gfx::unlock_surface`), `dword_4F49C8` put back; the clock on by `dword_5C2CF3`
//! (the recording's frame time) for the next.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); no flags read
//!
//! Caller: `drive::race_frame` (each frame drawn). Its calls through `part`: 0
//! `GetTickCount` (eax), 1 the lock (carry), 2 the writer, 3 the unlock; in the game the
//! system's clock, the platform's two (their hooks) and the writer's Rust version. Shadow mode runs the original alone (it writes a file). Self-test: a copy
//! whose four calls go to stand-ins (the lock failing or not), against the Rust version:
//! recording or not, the clock started or not, names with digits to carry; the variables
//! and the calls compared.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::mud::{copy, run_both};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::replay_screenshot", rva(0x41F3DB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RECORDING_RVA: usize = rva(0x5C2CCC); // dword_5C2CCC
const CLOCK_RVA: usize = rva(0x5C2CD0); // dword_5C2CD0
const STARTED_RVA: usize = rva(0x5C2CD4); // dword_5C2CD4
const NAME_RVA: usize = rva(0x5C2CDC); // "rpl\000000.pcx"
const NAME_LEN: usize = 15;
const FRAME_TIME_RVA: usize = rva(0x5C2CF3); // dword_5C2CF3 (unaligned)
const PICTURE_NAME_RVA: usize = rva(0x4F49C8); // dword_4F49C8: the name the writer uses

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn replay_screenshot(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        if game_read::<u32>(RECORDING_RVA) != 1 {
            return Ok(*entry);
        }
        let mut r = *entry;
        if game_read::<u32>(STARTED_RVA) != 1 {
            game_write(STARTED_RVA, 1u32);
            part(0, &mut r)?;
            game_write(CLOCK_RVA, r.eax);
        }
        let kept: u32 = game_read(PICTURE_NAME_RVA);
        r.eax = game_addr(NAME_RVA) as u32;
        r.ecx = 5;
        loop {
            r.ebx = r.ecx.wrapping_add(r.eax).wrapping_add(4);
            let digit = r.ebx as *mut u8;
            *digit = (*digit).wrapping_add(1);
            if *digit <= b'9' {
                break;
            }
            *digit = b'0';
            r.ecx = r.ecx.wrapping_sub(1);
            if (r.ecx as i32) < 0 {
                break;
            }
        }
        game_write(PICTURE_NAME_RVA, r.eax);
        let saved = (|| {
            part(1, &mut r)?;
            if !r.carry() {
                part(2, &mut r)?;
                part(3, &mut r)?;
            }
            Ok::<(), Refused>(())
        })();
        game_write(PICTURE_NAME_RVA, kept);
        saved?;
        game_write(
            CLOCK_RVA,
            game_read::<u32>(CLOCK_RVA).wrapping_add(game_read(FRAME_TIME_RVA)),
        );
    }
    Ok(*entry)
}

/// In the game: the system's clock, the platform's lock and unlock (their hooks) and the
/// PCX writer's Rust version (`gfx::write_screen_pcx`).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => r.eax = windows_sys::Win32::System::SystemInformation::GetTickCount(),
            2 => crate::gfx::write_screen_pcx::write_screen_pcx_in_game(r)?,
            p => {
                let at = [0, 0x4058AD, 0x411E0C, 0x405949][p as usize];
                call_game(r, game_addr(rva(at)));
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { replay_screenshot(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The test's lock fails (carry) while this is set.
static LOCK_FAILS: AtomicBool = AtomicBool::new(false);

/// The stand-ins: noted (`level_init::note`; the clock, place 0, gives `GIVES`); the lock's
/// carry by `LOCK_FAILS`.
fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    if place == 1 {
        r.set_carry(LOCK_FAILS.load(Ordering::SeqCst));
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp dword_5C2CCC, 1
    let copy = match copy(
        0x41F3DB,
        0x72,
        &[0x60, 0x83, 0x3D, 0xCC, 0x2C, 0x5C, 0x00, 0x01],
        &[0x1D, 0x4E, 0x55, 0x5A],
        &[(0x4D5DAC, 0), (0x4058AD, 1), (0x411E0C, 2), (0x405949, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let gives = crate::race::level_init::GIVES.swap(0x00C0_FFEE, Ordering::SeqCst);
    let regions: Vec<Region> = vec![
        at(RECORDING_RVA),
        at(CLOCK_RVA),
        at(STARTED_RVA),
        (game_addr(NAME_RVA) as u32, NAME_LEN),
        at(FRAME_TIME_RVA),
        at(PICTURE_NAME_RVA),
    ];
    let saved = capture(&regions);
    for (recording, started, digits, fails) in [
        (0u32, 0u32, *b"000000", false),
        (1, 0, *b"000000", false),
        (1, 1, *b"000009", false),
        (1, 1, *b"099999", false),
        (1, 1, *b"999999", false),
        (1, 1, *b"123456", true),
        (2, 1, *b"000000", false),
    ] {
        run_both(
            &mut t,
            &format!(
                "recording {recording}, started {started}, digits {}, lock fails {fails}",
                String::from_utf8_lossy(&digits)
            ),
            &copy,
            &regions,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(RECORDING_RVA, recording);
                game_write(STARTED_RVA, started);
                game_write(CLOCK_RVA, 1000u32);
                game_write(FRAME_TIME_RVA, 40u32);
                std::ptr::copy_nonoverlapping(
                    digits.as_ptr(),
                    (game_addr(NAME_RVA) + 4) as *mut u8,
                    6,
                );
                LOCK_FAILS.store(fails, Ordering::SeqCst);
            },
            &|r| unsafe { replay_screenshot(r, &mut test_part) },
        );
    }
    LOCK_FAILS.store(false, Ordering::SeqCst);
    crate::race::level_init::GIVES.store(gives, Ordering::SeqCst);
    restore(&regions, &saved);
    t.finish()
}
