//! `audio::thread` (0x409CCE): the audio thread's routine, from its start to `ExitThread`.
//!
//! In:  ebx, ecx, edx, edi, ebp = as the thread starter leaves them (carried into the
//!      commands: `audio::take_commands`)
//! Out: does not return (`ExitThread(0)`)
//!
//! Its slot (`stru_518604`, +0xC running): already running goes straight to the end. Else
//! running; "AUDIOTDB:START"; `CoInitialize`; `g_noAudio` set: the end. Else
//! `audio::take_sample_buffer`, `audio::start` (the carry: the end), `audio::read_splash`,
//! `audio::set_mode` with `g_menuSoundMode`, `threading::set_priority` 0xF; then a turn
//! every 20 ms: sleep what is left of them since the last (`g_audioTurnTick`; at least 1),
//! `audio::stream_command` and `audio::music_command` with edx 1 (ebx the last turn's
//! tick), `audio::take_commands`, `audio::update_voices`, `audio::restore_lost` when
//! `g_soundBufferLost`, until `g_quitting` is 1. The end: `g_noAudio` 1, `audio::close_all`,
//! `audio::free_sample_buffer`, `audio::write_splash`, `CoUninitialize`, "AUDIOTDB:QUIT",
//! priority 0, not running, `ExitThread(0)`.
//! ecx (and edx before the first turn) handed on to `take_commands` and `close_all` are, in
//! the original, what `CoInitialize` and the clock's Win32 calls leave; the Rust version
//! hands on the entry's. Neither is read there (ecx is a volume for action 1, which does not
//! read it; edx reaches `close_all` only when nothing plays yet, where every action stops).
//! The thunk is `ungated` (the thread is in it for the whole game). A refusal from a callee
//! hands the call to the original from its start: it finds the slot running and goes to
//! the end, so the sound stops (hook.log's "refused" line says why).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

const SLOT_RVA: usize = rva(0x518604); // stru_518604, the audio thread's slot
const TURN_TICK_RVA: usize = rva(0x4F82B8); // g_audioTurnTick
const START_TEXT: usize = rva(0x4DA5F2); // "AUDIOTDB:START"
const QUIT_TEXT: usize = rva(0x4DA601); // "AUDIOTDB:QUIT"

#[derive(Clone, Copy)]
pub struct ThreadVars {
    pub slot: u32,
    pub no_audio: u32,
    pub menu_mode: u32,
    pub turn_tick: u32,
    pub lost: u32,
    pub quitting: u32,
}

impl ThreadVars {
    pub fn game() -> Self {
        use crate::audio::voices::*;
        let a = |r| game_addr(r) as u32;
        Self {
            slot: a(SLOT_RVA),
            no_audio: a(crate::sound::queue::NO_AUDIO_RVA),
            menu_mode: a(crate::audio::choose_modes::MENU_MODE_RVA),
            turn_tick: a(TURN_TICK_RVA),
            lost: a(BUFFER_LOST_RVA),
            quitting: a(rva(0x51837C)),
        }
    }
}

/// Runs to the `ExitThread` (place 18), which in the game does not come back.
pub unsafe fn audio_thread(entry: &Regs, v: &ThreadVars) -> Result<(), Refused> {
    unsafe {
        let mut c = *entry;
        c.esi = v.slot;
        let running = (v.slot + 0xC) as *mut u8;
        let call = |place: u32, c: &mut Regs| -> Result<bool, Refused> {
            crate::audio::thread::game_part(place, c)
        };
        let started = running.read() != 1
            && {
                running.write(1);
                call(
                    0,
                    &mut Regs {
                        eax: game_addr(START_TEXT) as u32,
                        ..c
                    },
                )?;
                call(16, &mut c)?;
                read(v.no_audio) == 0
            }
            && {
                call(1, &mut c)?;
                !call(2, &mut c)?
            };
        if started {
            call(3, &mut c)?;
            c.eax = read(v.menu_mode);
            call(4, &mut c)?;
            c.eax = 0xF;
            call(5, &mut c)?;
            call(6, &mut c)?;
            write(v.turn_tick, c.eax);
            loop {
                c.ebx = read(v.turn_tick);
                call(6, &mut c)?;
                write(v.turn_tick, c.eax);
                // Converted: `rc2k_core::audio::device::turn_sleep`.
                c.eax = rc2k_core::audio::device::turn_sleep(c.ebx, c.eax);
                call(7, &mut c)?;
                c.edx = 1;
                call(8, &mut c)?;
                c.edx = 1;
                for place in 9..12 {
                    call(place, &mut c)?;
                }
                if read(v.lost) != 0 {
                    call(15, &mut c)?;
                }
                if read(v.quitting) == 1 {
                    break;
                }
            }
        }
        write(v.no_audio, 1);
        for place in [12, 13, 14, 17] {
            call(place, &mut c)?;
        }
        call(
            0,
            &mut Regs {
                eax: game_addr(QUIT_TEXT) as u32,
                ..c
            },
        )?;
        c.esi = v.slot;
        c.eax = 0;
        call(5, &mut c)?;
        running.write(0);
        call(18, &mut c)?;
    }
    Ok(())
}

/// The turn's work (places 8 .. 11 and 15) entered as a hook is: the original thread stopped
/// at its next hooked call while a self-test ran, and the Rust one calls the Rust versions
/// directly, so without this it ran on under the tests (`feed_stream`'s test saw
/// `g_lockedParts` change, 2026-10-15).
pub fn game_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    turn_part(place, r)
}

fn turn_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::*;
    unsafe {
        Ok(match place {
            0 => sound_device::game_part(1, r.eax)?,
            1 => {
                thread_parts::take_sample_buffer(&thread_parts::SampleBufferVars::game())?;
                false
            }
            2 => !start::start(r, &start::StartVars::game(), &mut start::game_part)?,
            3 | 14 => {
                a3d::splash(game_addr(a3d::SPLASH_KEPT_RVA) as u32)?;
                false
            }
            4 => !set_mode::set_mode(r, &set_mode::ModeVars::game(), &mut set_mode::game_part)?,
            5 => {
                crate::threading::set_priority::set_priority(r.eax as i32);
                false
            }
            6 => {
                r.eax = crate::system::tick_count::tick_count();
                false
            }
            7 => {
                crate::system::sleep::sleep(r.eax);
                false
            }
            8 => {
                stream_commands::stream_command(
                    r.eax,
                    r.ecx as i32,
                    r.edx,
                    game_addr(stream::SOUND_STREAM_RVA) as u32,
                    &mut stream_commands::game_part,
                )?;
                false
            }
            9 => {
                stream_commands::music_command(
                    r.eax,
                    r.ebx,
                    r.ecx as i32,
                    r.edx,
                    &stream_commands::MusicVars::game(),
                    &mut stream_commands::game_part,
                )?;
                false
            }
            10 => {
                take_commands::take_commands(r, &take_commands::Vars::game())?;
                false
            }
            11 => {
                update_voices::update_voices(&update_voices::Vars::game())?;
                crate::audio::stray_voices::check();
                false
            }
            12 => {
                close_all::close_all(r, game_addr(voices::SOUND_MODE_RVA) as u32)?;
                false
            }
            13 => {
                thread_parts::free_sample_buffer(&thread_parts::SampleBufferVars::game())?;
                false
            }
            15 => {
                restore::restore_lost(&restore::RestoreVars::game())?;
                false
            }
            16 => {
                windows_sys::Win32::System::Com::CoInitialize(std::ptr::null());
                false
            }
            17 => {
                windows_sys::Win32::System::Com::CoUninitialize();
                false
            }
            _ => windows_sys::Win32::System::Threading::ExitThread(0),
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { audio_thread(&entry, &ThreadVars::game()) }?;
    Ok(())
}
