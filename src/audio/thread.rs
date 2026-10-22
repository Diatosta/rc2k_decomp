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
//! the end, so the sound stops (hook.log's "refused" line says why). Shadow mode runs the
//! original alone. Self-test: a redirected copy whose calls all go to stand-ins (a clock
//! that is the test's, a sleep that waits for nothing, a quit after some turns) and whose
//! `ExitThread` goes back to the test, 12 cases.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::thread", rva(0x409CCE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SLOT_RVA: usize = rva(0x518604); // stru_518604, the audio thread's slot
const TURN_TICK_RVA: usize = rva(0x4F82B8); // g_audioTurnTick
const START_TEXT: usize = rva(0x4DA5F2); // "AUDIOTDB:START"
const QUIT_TEXT: usize = rva(0x4DA601); // "AUDIOTDB:QUIT"

/// The calls by place.
const CALLEES: [usize; 16] = [
    0x401439, // 0 log_write_l2_nl (eax)
    0x40B1D0, // 1 take_sample_buffer
    0x40A01D, // 2 start (carry)
    0x40C8CD, // 3 read_splash
    0x40A426, // 4 set_mode (eax)
    0x410D39, // 5 set_priority (eax)
    0x411024, // 6 tick_count (-> eax)
    0x41100C, // 7 sleep (eax)
    0x40C194, // 8 stream_command (ebx, edx)
    0x40C4DD, // 9 music_command (ebx, edx)
    0x40A8A4, // 10 take_commands (ebx, ecx, edx)
    0x40AC77, // 11 update_voices
    0x409DD5, // 12 close_all (ebx, ecx, edx)
    0x40B201, // 13 free_sample_buffer
    0x40C990, // 14 write_splash
    0x40A7C9, // 15 restore_lost
];
/// 16 `CoInitialize`, 17 `CoUninitialize`, 18 `ExitThread` (their thunks).
const CO_INITIALIZE: usize = 0x4D7CC2;
const CO_UNINITIALIZE: usize = 0x4D7CC8;
const EXIT_THREAD: usize = 0x4D5D10;
const CALLS: [usize; 22] = [
    0x1C, 0x24, 0x36, 0x3B, 0x42, 0x4C, 0x56, 0x5B, 0x6B, 0x85, 0x8F, 0x99, 0x9E, 0xA3, 0xC4, 0xC9,
    0xCE, 0xD3, 0xDF, 0xF0, 0xFB, 0x100,
];

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

/// A call (place, the registers there) -> its carry; place 6 leaves the tick in eax.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// Runs to the `ExitThread` (place 18), which in the game does not come back.
pub unsafe fn audio_thread(entry: &Regs, v: &ThreadVars, part: Part) -> Result<(), Refused> {
    unsafe {
        let mut c = *entry;
        c.esi = v.slot;
        let running = (v.slot + 0xC) as *mut u8;
        let mut call = |place: u32, c: &mut Regs| -> Result<bool, Refused> { part(place, c) };
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
    struct Gate;
    impl Drop for Gate {
        fn drop(&mut self) {
            crate::framework::hook_leave();
        }
    }
    let _gate = matches!(place, 8..=11 | 15).then(|| {
        crate::framework::hook_enter();
        Gate
    });
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
                take_commands::take_commands(
                    r,
                    &take_commands::Vars::game(),
                    &mut take_commands::game_part,
                )?;
                false
            }
            11 => {
                update_voices::update_voices(
                    &update_voices::Vars::game(),
                    &mut update_voices::game_part,
                )?;
                crate::audio::stray_voices::check();
                false
            }
            12 => {
                close_all::close_all(
                    r,
                    game_addr(voices::SOUND_MODE_RVA) as u32,
                    &mut close_all::game_part,
                )?;
                false
            }
            13 => {
                thread_parts::free_sample_buffer(&thread_parts::SampleBufferVars::game())?;
                false
            }
            15 => {
                restore::restore_lost(&restore::RestoreVars::game(), &mut restore::game_part)?;
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { audio_thread(&entry, &ThreadVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { audio_thread(&entry, &ThreadVars::game(), &mut game_part) }?;
    Ok(())
}

// ---- self-test ----

/// Each call: the place, eax / ebx / ecx / edx where it reads them.
static NOTED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());
/// The test's clock, its quit turn and the turns whose update loses a buffer.
static TICK: AtomicU32 = AtomicU32::new(0);
static TURNS: AtomicU32 = AtomicU32::new(0);
static QUIT_AFTER: AtomicU32 = AtomicU32::new(0);
static LOSE_ON: AtomicU32 = AtomicU32::new(0);
static START_FAILS: AtomicU32 = AtomicU32::new(0);
/// The test's quitting and lost flags (their addresses).
static FLAGS: AtomicU32 = AtomicU32::new(0);

fn test_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    let eax = if matches!(place, 0 | 4 | 5 | 7) {
        r.eax
    } else {
        0
    };
    let ebx = if matches!(place, 8 | 9 | 10 | 12) {
        r.ebx
    } else {
        0
    };
    // ecx is never compared, nor edx where the original has not loaded it (`close_all`):
    // after `CoInitialize` and the clock's Win32 calls they are what Windows leaves.
    let ecx = 0;
    let edx = if matches!(place, 8..=10) { r.edx } else { 0 };
    NOTED.lock().unwrap().push([place, eax, ebx, ecx, edx]);
    let flags = FLAGS.load(Ordering::SeqCst);
    match place {
        2 => return Ok(START_FAILS.load(Ordering::SeqCst) == 1),
        6 => {
            // 7, 9, 31, 33, .. ms apart: some turns left to wait, some late.
            let n = TICK.fetch_add(1, Ordering::SeqCst);
            r.eax = 1000 + n * 13 + if n % 2 == 1 { 9 } else { 0 };
        }
        11 => {
            let turn = TURNS.fetch_add(1, Ordering::SeqCst) + 1;
            unsafe {
                if turn == LOSE_ON.load(Ordering::SeqCst) {
                    write(flags + 4, 1);
                }
                if turn >= QUIT_AFTER.load(Ordering::SeqCst) {
                    write(flags, 1);
                }
            }
        }
        15 => unsafe { write(flags + 4, 0) },
        _ => {}
    }
    Ok(false)
}

fn handler(place: u32, r: &mut Regs) {
    let carry = test_part(place, r).unwrap_or(false);
    r.set_carry(carry);
}

extern "system" fn co_initialize(_: u32) -> i32 {
    NOTED.lock().unwrap().push([16, 0, 0, 0, 0]);
    0
}

extern "system" fn co_uninitialize() {
    NOTED.lock().unwrap().push([17, 0, 0, 0, 0]);
}

/// The copy's entry registers (ebx, ecx, edx) and the stack to go back to.
static ENTRY: [AtomicU32; 3] = [const { AtomicU32::new(0) }; 3];
static SAVED: AtomicU32 = AtomicU32::new(0);

/// Runs the copy at `addr` with `ENTRY`'s registers; it ends in `exit_thread`.
#[unsafe(naked)]
unsafe extern "C" fn run_copy(addr: u32) {
    core::arch::naked_asm!(
        "pushad",
        "pushfd",
        "mov [{saved}], esp",
        "mov eax, [esp + 40]",
        "mov ebx, [{entry}]",
        "mov ecx, [{entry} + 4]",
        "mov edx, [{entry} + 8]",
        "call eax",
        "ud2",
        saved = sym SAVED,
        entry = sym ENTRY,
    )
}

/// `ExitThread` in the copy: notes it and goes back to `run_copy`'s caller.
#[unsafe(naked)]
unsafe extern "C" fn exit_thread() {
    core::arch::naked_asm!(
        "push 18",
        "call {note}",
        "add esp, 4",
        "mov esp, [{saved}]",
        "popfd",
        "popad",
        "ret",
        note = sym note_exit,
        saved = sym SAVED,
    )
}

extern "C" fn note_exit(place: u32) {
    NOTED.lock().unwrap().push([place, 0, 0, 0, 0]);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // 0 .. 3 the slot, 4 no audio, 5 menu mode, 6 turn tick, 7 quitting, 8 lost.
    let mut m = vec![0u32; 9];
    let at = m.as_mut_ptr() as u32;
    let a = |n: u32| at + n * 4;
    let v = ThreadVars {
        slot: a(0),
        no_audio: a(4),
        menu_mode: a(5),
        turn_tick: a(6),
        quitting: a(7),
        lost: a(8),
    };
    FLAGS.store(v.quitting, Ordering::SeqCst);
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x409CCE,
            len: 0x107,
            entry: &[0xFC, 0x8D, 0x35, 0x04, 0x86, 0x51, 0x00],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|callee| {
            if callee == rva(CO_INITIALIZE) {
                return Some(co_initialize as *const () as usize);
            }
            if callee == rva(CO_UNINITIALIZE) {
                return Some(co_uninitialize as *const () as usize);
            }
            if callee == rva(EXIT_THREAD) {
                return Some(exit_thread as *const () as usize);
            }
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x518604, v.slot),
            (0x4F82B4, v.no_audio),
            (0x4F82C4, v.menu_mode),
            (0x4F82B8, v.turn_tick),
            (0x51837C, v.quitting),
            (0x4F82CC, v.lost),
        ],
        10,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let entry = Regs {
        ebx: 0xB0B0,
        ecx: 0xC0C0,
        edx: 0xD0D0,
        ..Regs::default()
    };
    for (n, x) in [entry.ebx, entry.ecx, entry.edx].iter().enumerate() {
        ENTRY[n].store(*x, Ordering::SeqCst);
    }
    // (running already, no audio, start fails, quit after, lose on)
    let cases = [
        (false, false, false, 1u32, 0u32),
        (false, false, false, 3, 0),
        (false, false, false, 5, 2),
        (false, false, false, 4, 4),
        (false, false, true, 3, 0),
        (false, true, false, 3, 0),
        (true, false, false, 3, 0),
        (false, false, false, 8, 1),
        (false, false, false, 2, 2),
        (true, true, true, 1, 0),
        (false, false, false, 6, 3),
        (false, false, false, 7, 7),
    ];
    for (running, no_audio, start_fails, quit_after, lose_on) in cases {
        let run = |rust: bool| {
            unsafe {
                let p = at as *mut u32;
                for n in 0..9 {
                    p.add(n).write_volatile(0);
                }
                (p as *mut u8).add(0xC).write_volatile(running as u8);
                p.add(4).write_volatile(no_audio as u32);
                p.add(5).write_volatile(0x11221);
            }
            TICK.store(0, Ordering::SeqCst);
            TURNS.store(0, Ordering::SeqCst);
            QUIT_AFTER.store(quit_after, Ordering::SeqCst);
            LOSE_ON.store(lose_on, Ordering::SeqCst);
            START_FAILS.store(start_fails as u32, Ordering::SeqCst);
            NOTED.lock().unwrap().clear();
            let done = if rust {
                unsafe { audio_thread(&entry, &v, &mut test_part) }.is_ok()
            } else {
                unsafe { run_copy(c.addr() as u32) };
                true
            };
            (
                done,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                unsafe { std::slice::from_raw_parts(at as *const u32, 9) }.to_vec(),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!(
                "running {running}, no audio {no_audio}, start fails {start_fails}, quit after {quit_after}, lose on {lose_on}: original {:x?} {:x?}; rust {:x?} {:x?}",
                theirs.1, theirs.2, mine.1, mine.2
            )
        });
    }
    drop(m);
    t.finish()
}
