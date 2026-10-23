//! The calls of `start`'s way out (0x4010EE on, ported 2026-10-19) and of the shutdown
//! thread, by place: the game's functions by their IDA address (in the game the Rust
//! versions; in the tests the noting stand-ins of `sound::turn_parts`), the Win32 functions
//! by the places below (in the game the functions themselves, in the tests
//! `system::api_stand_ins`).

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::system::api_stand_ins::Api;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

// The game's functions.
pub const FREE_LIST: u32 = 0x4028F5; // memory::free_list (ebx)
pub const VIRTUAL_FREE: u32 = 0x402B3B; // memory::virtual_free (eax, if not 0)
pub const LOG_2: u32 = 0x401432; // logfile::log_write_l2
pub const LOG_2_NL: u32 = 0x401439; // logfile::log_write_l2_nl
pub const HEX8_2_NL: u32 = 0x401634; // logfile::log_hex8_l2_nl
pub const WAIT_FOR_THREADS: u32 = 0x410BA8; // threading::wait_for_threads
// threading::resume_slot_thread

// The Win32 functions (`Api` places).
pub const MESSAGE_BOX: u32 = 0; // MessageBoxA(hwnd, text, caption, type)
pub const SET_ERROR_MODE: u32 = 1; // SetErrorMode(mode)
pub const POST_MESSAGE: u32 = 2; // PostMessageA(hwnd, msg, wparam, lparam)
pub const EXIT_THREAD: u32 = 3; // ExitThread(code)
pub const WAIT_MULTIPLE: u32 = 4; // WaitForMultipleObjects(n, handles, all, ms)
pub const CLOSE_HANDLE: u32 = 5; // CloseHandle(handle)
pub const RESUME_THREAD: u32 = 6; // ResumeThread(thread)

/// The game's calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_hex8::log_hex8, log_write::log_write};
    unsafe {
        match place {
            FREE_LIST => {
                crate::memory::free_list::free_list(r.ebx as *mut _)?;
                r.set_carry(false);
            }
            VIRTUAL_FREE => {
                if r.eax != 0 {
                    use windows_sys::Win32::System::Memory::{MEM_RELEASE, VirtualFree};
                    VirtualFree(r.eax as _, 0, MEM_RELEASE);
                }
            }
            LOG_2 => log_write(r.eax as *const u8, false, 2),
            LOG_2_NL => log_write(r.eax as *const u8, true, 2),
            HEX8_2_NL => log_hex8(r.eax, true, 2),
            WAIT_FOR_THREADS => {
                r.eax = crate::threading::wait_for_threads::wait_for_threads(&mut game_api)
            }
            _ => return Err(BadInput("a call of the way out with no version").into()),
        }
    }
    Ok(())
}

/// The Win32 calls in the game.
pub fn game_api(place: u32, args: &[u32]) -> u32 {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::Debug::SetErrorMode;
    use windows_sys::Win32::System::Threading::{ExitThread, ResumeThread, WaitForMultipleObjects};
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxA, PostMessageA};
    let x = |n: usize| args.get(n).copied().unwrap_or(0);
    unsafe {
        match place {
            MESSAGE_BOX => MessageBoxA(x(0) as _, x(1) as _, x(2) as _, x(3)) as u32,
            SET_ERROR_MODE => SetErrorMode(x(0)),
            POST_MESSAGE => PostMessageA(x(0) as _, x(1), x(2) as _, x(3) as _) as u32,
            EXIT_THREAD => ExitThread(x(0)),
            WAIT_MULTIPLE => WaitForMultipleObjects(x(0), x(1) as _, x(2) as _, x(3)),
            CLOSE_HANDLE => CloseHandle(x(0) as _) as u32,
            RESUME_THREAD => ResumeThread(x(0) as _),
            _ => 0,
        }
    }
}

/// A Rust version of the way out: the registers in and out, the game's calls, the Win32
/// calls.
pub type Rust = fn(&mut Regs, Part, Api) -> Result<(), Refused>;

/// `dispatch_exit` outside shadow mode, without its hook (a refusal is the caller's).
pub fn exit_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    let mut g = *r;
    rust(&mut g, &mut game_part, &mut game_api)?;
    *r = Regs { esp: r.esp, ..g };
    Ok(())
}
