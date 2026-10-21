//! `system::read_command_line` (0x410684): takes the switches off the command line.
//!
//! In:  nothing
//! Out: every register preserved
//! The first word is the program (its place and length to `dword_518BA8`, `dword_518BAC`).
//! Of the words after it only those of two characters or more count, by their first two:
//! - `-v`: a message box with the version ("Rally 5.30 (4.8.2000)", a tab, a line break and
//!   the second version text `byte_4E0077`, built in the fatal report's `Text`; the title
//!   is the report's "Rally: Guru Meditation" cut after "Rally"), and the game leaves
//!   through `start`'s exit path (0x401116) without coming back here. The words after
//!   it are not read;
//! - `-u`: `dword_51B9CC` = 1; `-p`: `g_installing` (0x51B9E0) = 1; `-a`: `g_noAudio` = 1;
//! - `-i`: `dword_51B9D0` and `dword_51BA04` = 1, the next two words are copied (without a
//!   NUL) to `byte_95E6F8` and `byte_95E7F8`, and the first characters of the two after
//!   them, less `0`, go to `dword_51BA08` and `dword_51BA0C` (0 for a word that is not
//!   there);
//! - `-c`: the next word's first character: `1` sets `dword_5AB954` (`ignoresecondary`),
//!   `2` sets `dword_5AB950` (`ignoreprimary`).
//!
//! The line ends at a word of no length that starts where the reading stands.
//! One caller, `start`. Refuses a word for `-i` of 0x100 bytes or more.
//! Shadow mode reads the line twice and compares, but with `-v` the original runs alone
//! (one box). The `-v` part cannot be run in a test: it was looked at by the owner.
//! Manual patch with a private copy (0x126 bytes; eight calls and the jump to the `-v`
//! part relocated): a call starts in the first bytes.
//! Self-test: a redirected copy that gets its line from the test.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use windows_sys::Win32::System::Environment::GetCommandLineA;
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::read_command_line", rva(0x410684), thunk, &ORIGINAL)
        .private_copy(0x126)
        .relocate_calls(&[0x01, 0x06, 0x18, 0x35, 0x7D, 0x8E, 0x9F, 0xB6, 0xD9])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PROGRAM_RVA: usize = rva(0x518BA8); // dword_518BA8
const PROGRAM_LEN_RVA: usize = rva(0x518BAC); // dword_518BAC
const SWITCH_U_RVA: usize = rva(0x51B9CC); // dword_51B9CC
const INSTALLING_RVA: usize = rva(0x51B9E0); // g_installing
const SWITCH_I_RVA: usize = rva(0x51B9D0); // dword_51B9D0
const SWITCH_I_TOO_RVA: usize = rva(0x51BA04); // dword_51BA04
const I_FIRST_RVA: usize = rva(0x95E6F8); // byte_95E6F8
const I_SECOND_RVA: usize = rva(0x95E7F8); // byte_95E7F8
const I_WORD_ROOM: usize = 0x100;
const I_THIRD_RVA: usize = rva(0x51BA08); // dword_51BA08
const I_FOURTH_RVA: usize = rva(0x51BA0C); // dword_51BA0C
const IGNORE_SECONDARY_RVA: usize = rva(0x5AB954); // dword_5AB954
const IGNORE_PRIMARY_RVA: usize = rva(0x5AB950); // dword_5AB950
const NO_AUDIO_RVA: usize = rva(0x4F82B4); // g_noAudio
const NEXT_ARGUMENT: usize = rva(0x4107B8);
const VERSION_RVA: usize = rva(0x4E0000); // "$VER: Rally 5.30 (4.8.2000)"
const SECOND_TEXT_RVA: usize = rva(0x4E0077); // byte_4E0077
const REPORT_TEXT_RVA: usize = rva(0x9526F8); // Text
const CAPTION_RVA: usize = rva(0x4F66FD); // "Rally: Guru Meditation"
const CAPTION_END_RVA: usize = rva(0x4F6702); // byte_4F6702: its `:`
const EXIT_PATH_RVA: usize = rva(0x401116);

// ---- Rust version ----

/// What the line asks for besides its switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Nothing,
    /// `-v`: the version is to be shown and the game left.
    Version,
}

/// The `-v` part: the box, and out through `start`'s exit path. Does not return.
pub unsafe fn show_version_and_leave(regs: &Regs) -> ! {
    unsafe {
        game_write(CAPTION_END_RVA, 0u8);
        let c_text = |at: usize| std::ffi::CStr::from_ptr(game_addr(at) as *const _).to_bytes();
        let mut text = c_text(VERSION_RVA + 6).to_vec();
        text.extend(*b"\t\r\n");
        text.extend(c_text(SECOND_TEXT_RVA));
        text.push(0);
        let place = game_addr(REPORT_TEXT_RVA) as *mut u8;
        std::ptr::copy(text.as_ptr(), place, text.len());
        MessageBoxA(
            std::ptr::null_mut(),
            place,
            game_addr(CAPTION_RVA) as *const u8,
            0x2040,
        );
        let mut regs = *regs;
        call_game(&mut regs, game_addr(EXIT_PATH_RVA));
        // The exit path ends in `ExitProcess`.
        std::process::abort()
    }
}

pub unsafe fn read_command_line(line: *const u8) -> Result<Asked, BadInput> {
    // Converted: `rc2k_core::system::command_line::read`, the whole line read before
    // anything is written (a refused `-i` word now leaves nothing written).
    unsafe {
        let sw = rc2k_core::system::command_line::read(&crate::bridge::Raw::new(line))?;
        game_write(PROGRAM_RVA, line.wrapping_add(sw.program.0) as u32);
        game_write(PROGRAM_LEN_RVA, sw.program.1 as u32);
        for (on, place) in [
            (sw.u, SWITCH_U_RVA),
            (sw.installing, INSTALLING_RVA),
            (sw.no_audio, NO_AUDIO_RVA),
        ] {
            if on {
                game_write(place, 1u32);
            }
        }
        for i in &sw.install {
            game_write(SWITCH_I_RVA, 1u32);
            game_write(SWITCH_I_TOO_RVA, 1u32);
            for (place, (start, len)) in [I_FIRST_RVA, I_SECOND_RVA].into_iter().zip(i.words) {
                std::ptr::copy(line.wrapping_add(start), game_addr(place) as *mut u8, len);
            }
            game_write(I_THIRD_RVA, i.digits[0]);
            game_write(I_FOURTH_RVA, i.digits[1]);
        }
        if sw.ignore_secondary {
            game_write(IGNORE_SECONDARY_RVA, 1u32);
        }
        if sw.ignore_primary {
            game_write(IGNORE_PRIMARY_RVA, 1u32);
        }
        Ok(if sw.version {
            Asked::Version
        } else {
            Asked::Nothing
        })
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = [
        PROGRAM_RVA,
        PROGRAM_LEN_RVA,
        SWITCH_U_RVA,
        INSTALLING_RVA,
        SWITCH_I_RVA,
        SWITCH_I_TOO_RVA,
        I_THIRD_RVA,
        I_FOURTH_RVA,
        IGNORE_SECONDARY_RVA,
        IGNORE_PRIMARY_RVA,
        NO_AUDIO_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((game_addr(I_FIRST_RVA) as u32, I_WORD_ROOM));
    regions.push((game_addr(I_SECOND_RVA) as u32, I_WORD_ROOM));
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let line = unsafe { GetCommandLineA() };
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { read_command_line(line) };
    if !shadow() {
        match mine {
            Ok(Asked::Nothing) => {}
            Ok(Asked::Version) => unsafe { show_version_and_leave(r) },
            Err(e) => {
                restore(&regions, &before);
                unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            }
        }
        return;
    }
    let left = capture(&regions);
    restore(&regions, &before);
    let mut o = *r;
    // With `-v` this shows the box and does not come back.
    unsafe { call_original(&mut o) };
    let theirs = capture(&regions);
    if mine != Ok(Asked::Nothing) || theirs != left || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            let first = theirs.iter().zip(&left).position(|(a, b)| a != b);
            format!(
                "line={:?}: the original came back; rust {mine:?}; first different byte {first:?}",
                unsafe { std::ffi::CStr::from_ptr(line as *const _) }
            )
        });
    }
}

// ---- self-test: a copy of the original that gets its line from the test ----

static TEST_LINE: AtomicU32 = AtomicU32::new(0);

extern "C" fn test_line() -> u32 {
    TEST_LINE.load(Ordering::SeqCst)
}

const LEN: usize = 0x126;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x92, 0x56, 0x0C, 0x00, 0xE8, 0x29];
const CALLS: [usize; 8] = [0x01, 0x06, 0x18, 0x7D, 0x8E, 0x9F, 0xB6, 0xD9];
const LINES: [&[u8]; 12] = [
    b"\0",
    b"ral.exe\0",
    b"\"C:\\Program Files (x86)\\RC2k\\RAL.EXE\" \0",
    b"ral.exe -a -u -p\0",
    b"ral.exe -audio x -c 1 -c 2 -c 3 -c\0",
    b"ral.exe -i first \"second word\" 3 7 -a\0",
    b"ral.exe -i only\0",
    b"ral.exe -i a b x\0",
    b"ral.exe - a -A \"-a\" \"\" -u\0",
    b"ral.exe -c \"\" -c\t2\r-p\0",
    b"  ral.exe  \t -x -uu  \0",
    b"ral.exe -i \"\" \"\" \"\" \"\" -c 12\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The one call that is not `next_argument` is `GetCommandLineA`.
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x410684)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee != NEXT_ARGUMENT).then_some(test_line as *const () as usize),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    for line in LINES {
        TEST_LINE.store(line.as_ptr() as u32, Ordering::SeqCst);
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            let done = if rust {
                read_command_line(line.as_ptr()) == Ok(Asked::Nothing)
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (done, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "line={:?}: original done={} rust done={}; first different byte {first:?} (eleven dwords, then the two words of -i)",
                String::from_utf8_lossy(line),
                theirs.0,
                mine.0
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
