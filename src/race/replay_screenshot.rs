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
//! Caller: `drive::race_frame` (each frame drawn).

use crate::errors::Refused;
use crate::framework::rva;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::picture_store::Slot as PicSlot;

const RECORDING_RVA: usize = rva(0x5C2CCC); // dword_5C2CCC
const CLOCK_RVA: usize = rva(0x5C2CD0); // dword_5C2CD0
const STARTED_RVA: usize = rva(0x5C2CD4); // dword_5C2CD4
const NAME_RVA: usize = rva(0x5C2CDC); // "rpl\000000.pcx"
const FRAME_TIME_RVA: usize = rva(0x5C2CF3); // dword_5C2CF3 (unaligned)
const PICTURE_NAME_RVA: PicSlot = PicSlot::at(0x4F49C8); // dword_4F49C8: the name the writer uses

/// The Rust version (see the header). (The original keeps every register.)
pub unsafe fn replay_screenshot() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(RECORDING_RVA) != 1 {
            return Ok(());
        }
        if game_read::<u32>(STARTED_RVA) != 1 {
            game_write(STARTED_RVA, 1u32);
            game_write(
                CLOCK_RVA,
                windows_sys::Win32::System::SystemInformation::GetTickCount(),
            );
        }
        let kept: u32 = game_read(PICTURE_NAME_RVA);
        let name = game_addr(NAME_RVA) as u32;
        // Converted: `rc2k_core::race::viewer::count_up` (the name's six digits from +4).
        let digits = (name + 4) as *mut [u8; 6];
        let (counted, _, _) = rc2k_core::race::viewer::count_up(digits.read());
        digits.write(counted);
        game_write(PICTURE_NAME_RVA, name);
        let saved = (|| {
            // The surface locked (the carry says it was not), the picture written, the
            // surface unlocked.
            if crate::platform::lock_surface() >= 1 {
                crate::gfx::write_screen_pcx::write_screen_pcx()?;
                crate::platform::unlock_surface();
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
    Ok(())
}
