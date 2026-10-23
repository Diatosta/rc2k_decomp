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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

const RECORDING_RVA: usize = rva(0x5C2CCC); // dword_5C2CCC
const CLOCK_RVA: usize = rva(0x5C2CD0); // dword_5C2CD0
const STARTED_RVA: usize = rva(0x5C2CD4); // dword_5C2CD4
const NAME_RVA: usize = rva(0x5C2CDC); // "rpl\000000.pcx"
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
        // Converted: `rc2k_core::race::viewer::count_up` (the name's six digits from +4).
        let digits = (r.eax + 4) as *mut [u8; 6];
        let (counted, last, ecx) = rc2k_core::race::viewer::count_up(digits.read());
        digits.write(counted);
        r.ebx = r.eax + 4 + last as u32;
        r.ecx = ecx;
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
                crate::calls::call(rva(at), r);
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { replay_screenshot(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
