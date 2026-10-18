//! `fatal::report` (0x40763C): the fatal-error and breakpoint reporter. About 190 functions
//! jump here, each from a small stub that has stored a code in `g_fatalCode` (0x4F52A4) and a message
//! pointer in `g_fatalMessage` (0x4F52A8).
//!
//! In:  entered by `jmp`, with the registers and flags of the place that failed
//! Out: for a code without bit 31, returns (to whoever called the function that jumped here),
//!      with every register as it was; for a code with bit 31, does not return
//! Only the first report of a run is written (`g_fatalReported`, 0x4F67BC). It builds this text in `Text`
//! (0x9526F8, 0x400 bytes):
//!   unless the code has bit 28: "Error code [XXXXXXXX]" (bit 31) or "Breakpoint [XXXXXXXX]";
//!   the message, if there is one, with the two characters `\n` turned into a line break;
//!   the string at 0x4F67BD;
//!   unless the code has bit 28: "Last file: `path'" if a file is open, "W32=XXXXXXXX" if
//!   `dword_4E0598` is not 0, and the registers and five flags, which are written into the
//!   template at 0x4F673A first.
//! Then `g_logEnabled` = 1 and "Guru Meditation" and the text go to the log.
//! After that, also when it was not the first report, a code with bit 31 asks the game to
//! quit (`threading::request_quit`, 0x410BF9) and jumps: to the exit path in `start` (0x4010EE) if the current
//! thread has no thread slot (esi is then 0), else to that slot's start routine, with esi =
//! the slot.
//!
//! The thunk is written by hand: the function is entered by a jump and may leave by one. It
//! reserves a stack slot that `dispatch` fills with where to go, and ends in a `ret` through
//! that slot: a stub that returns, the game's exit path, a thread routine, or the original.
//! Everything it calls is ported; only its two ways out of a fatal error are jumps into
//! original code.
//! Refuses (the original then runs): a text that does not fit `Text`.
//! Shadow mode cannot run both: the original runs, and what the Rust version would have
//! written is put in hook.log beforehand, to compare with the game's log by eye. The
//! self-test runs a copy of the original whose log, quit and thread-slot calls go to
//! stand-ins and whose two ways out land in the test.

use crate::errors::BadInput;
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write, log, rva,
    shadow,
};
use crate::menu::text_entry::log_l1;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use crate::strings::bin_append::bin_append;
use crate::strings::hex_append::hex_append;
use crate::strings::utils::bytes_str;
use crate::threading::find_thread_slot::find_thread_slot;
use crate::threading::request_quit::request_quit;
use std::ffi::{CStr, c_char};
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("fatal::report", rva(0x40763C), thunk, &ORIGINAL).self_test(self_test);

const _: () = assert!(size_of::<Regs>() == 36);

/// Detour target. The stack slot pushed first is filled by `dispatch` with the address the
/// final `ret` goes to; at that point every register and flag is as it was on entry (or as
/// `dispatch` set it) and the stack is as the game left it.
#[unsafe(naked)]
unsafe extern "C" fn thunk() {
    core::arch::naked_asm!(
        "push 0",              // where to go: dispatch fills it in
        "pushfd",              // Regs.eflags
        "pushad",
        "cld",
        "mov eax, esp",
        "push eax",            // arg: *mut Regs
        "call {dispatch}",
        "add esp, 4",
        "popad",
        "popfd",
        "ret",
        dispatch = sym dispatch,
    )
}

/// Where a non-fatal report goes: it leaves the flags of the original's last `test` (the
/// fatal bit, which is clear) and returns on the game's stack.
#[unsafe(naked)]
unsafe extern "C" fn return_to_game() {
    core::arch::naked_asm!("test dword ptr [{zero}], 0x80000000", "ret", zero = sym ZERO)
}

static ZERO: AtomicU32 = AtomicU32::new(0);

// ---- game data ----

pub const CODE_RVA: usize = rva(0x4F52A4); // g_fatalCode
pub const MESSAGE_RVA: usize = rva(0x4F52A8); // g_fatalMessage
const REPORTED_RVA: usize = rva(0x4F67BC); // g_fatalReported: 1 once a report was written
const EXTRA_RVA: usize = rva(0x4F67BD); // `cursor` in IDA: a string added to every report
const W32_RVA: usize = rva(0x4E0598); // dword_4E0598
const TEMPLATE_RVA: usize = rva(0x4F673A); // aEaxXxxxxxxxEbx
const TEXT_RVA: usize = rva(0x9526F8); // Text
const TEXT_LEN: usize = 0x400;
const LOG_ENABLED_RVA: usize = rva(0x4E0094); // g_logEnabled
const GURU_RVA: usize = rva(0x4F6704); // aGuruMeditation + 1: "Guru Meditation"
const QUIT_RVA: usize = rva(0x410BF9); // threading::request_quit: asks the game to quit, once
const EXIT_PATH_RVA: usize = rva(0x4010EE); // loc_4010EE in `start`

const FATAL: u32 = 0x8000_0000;
const TEXT_ONLY: u32 = 0x1000_0000;

const TEMPLATE: &[u8] = b"EAX=XXXXXXXX\tEBX=XXXXXXXX\tECX=XXXXXXXX\tEDX=XXXXXXXX\r\nESI=XXXXXXXX\tEDI=XXXXXXXX\tESP=XXXXXXXX\tEBP=XXXXXXXX\r\nC=X  Z=X  S=X  V=X  P=X";
/// Where the eight register values go in the template, and where ESP's does.
const HEX_AT: [usize; 8] = [4, 17, 30, 43, 57, 70, 83, 96];
const ESP_AT: usize = 83;
/// Where the five flags go, with their bits: carry, zero, sign, overflow, parity.
const FLAG_AT: [(usize, u32); 5] = [(108, 1), (113, 0x40), (118, 0x80), (123, 0x800), (128, 4)];

// ---- Rust version ----

/// The registers and flags at the place that failed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub esp: u32,
    pub ebp: u32,
    pub flags: u32,
}

/// The register and flag lines.
pub fn register_dump(c: &Context) -> Vec<u8> {
    let mut dump = TEMPLATE.to_vec();
    let values = [c.eax, c.ebx, c.ecx, c.edx, c.esi, c.edi, c.esp, c.ebp];
    for (at, value) in HEX_AT.into_iter().zip(values) {
        // Eight digits are always accepted.
        if let Ok(hex) = hex_append(value, 8) {
            dump[at..at + 8].copy_from_slice(&hex);
        }
    }
    for (at, bit) in FLAG_AT {
        if let Ok(digit) = bin_append((c.flags & bit != 0) as u32, 1) {
            dump[at] = digit[0];
        }
    }
    dump
}

/// The report, without its NUL. `message` and `last_file` exclude their NUL; an empty
/// message is as good as none.
pub fn report_text(
    code: u32,
    message: Option<&[u8]>,
    extra: &[u8],
    last_file: Option<&[u8]>,
    w32: u32,
    context: &Context,
) -> Vec<u8> {
    let mut text = Vec::new();
    let full = code & TEXT_ONLY == 0;
    if full {
        text.extend_from_slice(if code & FATAL != 0 {
            b"Error code"
        } else {
            b"Breakpoint"
        });
        text.extend_from_slice(b" [");
        text.extend(hex_append(code, 8).unwrap_or_default());
        text.extend_from_slice(b"]\r\n");
    }
    if let Some(message) = message.filter(|m| !m.is_empty()) {
        let mut rest = message;
        while let [c, tail @ ..] = rest {
            if *c == b'\\' && tail.first() == Some(&b'n') {
                text.extend_from_slice(b"\r\n");
                rest = &tail[1..];
            } else {
                text.push(*c);
                rest = tail;
            }
        }
        text.extend_from_slice(b"\r\n");
    }
    text.extend_from_slice(extra);
    if full {
        if let Some(path) = last_file {
            text.extend_from_slice(b"Last file: `");
            text.extend_from_slice(path);
            text.extend_from_slice(b"'\r\n");
        }
        if w32 != 0 {
            text.extend_from_slice(b"W32=");
            text.extend(hex_append(w32, 8).unwrap_or_default());
            text.extend_from_slice(b"\r\n");
        }
        text.extend(register_dump(context));
    }
    text
}

unsafe fn cstr<'a>(ptr: *const u8) -> &'a [u8] {
    unsafe { CStr::from_ptr(ptr as *const c_char) }.to_bytes()
}

/// The report for what the game's variables hold now.
unsafe fn text_from_game(context: &Context) -> Vec<u8> {
    unsafe {
        let message: u32 = game_read(MESSAGE_RVA);
        let last_file: u32 = game_read(LAST_FILE_PATH_RVA);
        report_text(
            game_read(CODE_RVA),
            (message != 0).then(|| cstr(message as *const u8)),
            cstr(game_ptr(EXTRA_RVA)),
            (last_file != 0).then(|| cstr(last_file as *const u8)),
            game_read(W32_RVA),
            context,
        )
    }
}

/// How the reporter leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leave {
    /// Back to the game: the code was not fatal.
    Return,
    /// To the exit path in `start`, with esi = 0: a fatal code on a thread without a slot.
    Exit,
    /// To the start routine of the current thread's slot, with esi = the slot.
    Routine { slot: u32, routine: u32 },
}

/// What the reporter needs from code that is not ported, or from a test.
pub struct Outside<'a> {
    /// Logs a NUL-terminated text at level 1 with a line break.
    pub log: &'a mut dyn FnMut(*const u8),
    /// Asks the game to quit.
    pub quit: &'a mut dyn FnMut(),
    /// The current thread's slot, or 0.
    pub thread_slot: &'a dyn Fn() -> u32,
}

pub unsafe fn report(context: &Context, outside: &mut Outside) -> Result<Leave, BadInput> {
    unsafe {
        let code: u32 = game_read(CODE_RVA);
        let text = text_from_game(context);
        if text.len() + 1 > TEXT_LEN {
            return Err(BadInput("the report does not fit its buffer"));
        }
        let reported = AtomicU8::from_ptr(game_ptr(REPORTED_RVA));
        if reported.swap(1, Ordering::SeqCst) != 1 {
            if code & TEXT_ONLY == 0 {
                // The original fills in its template in place, so it keeps the values.
                let dump = register_dump(context);
                std::ptr::copy_nonoverlapping(dump.as_ptr(), game_ptr(TEMPLATE_RVA), dump.len());
            }
            let buffer: *mut u8 = game_ptr(TEXT_RVA);
            std::ptr::copy_nonoverlapping(text.as_ptr(), buffer, text.len());
            buffer.add(text.len()).write(0);
            game_write(LOG_ENABLED_RVA, 1u32);
            (outside.log)(game_ptr(GURU_RVA));
            (outside.log)(buffer);
        }
        if code & FATAL == 0 {
            return Ok(Leave::Return);
        }
        (outside.quit)();
        let slot = (outside.thread_slot)();
        if slot == 0 {
            return Ok(Leave::Exit);
        }
        Ok(Leave::Routine {
            slot,
            routine: (slot as *const u32).read_unaligned(),
        })
    }
}

// ---- hook ----

/// The registers and flags the thunk saved, as the place that failed had them. The thunk
/// pushed two dwords before `pushad`, so the stack pointer was 8 higher.
fn context_of(r: &Regs) -> Context {
    Context {
        eax: r.eax,
        ebx: r.ebx,
        ecx: r.ecx,
        edx: r.edx,
        esi: r.esi,
        edi: r.edi,
        esp: r.esp.wrapping_add(8),
        ebp: r.ebp,
        flags: r.eflags,
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let context = context_of(r);
    let original = ORIGINAL.load(Ordering::SeqCst) as u32;
    log(&format!(
        "fatal::report: {}",
        crate::framework::crash_note()
    ));

    let target = if shadow() {
        let text = unsafe { text_from_game(&context) };
        log(&format!(
            "fatal::report: the original runs; the Rust version would write: {:?}",
            String::from_utf8_lossy(&text)
        ));
        original
    } else {
        let mut outside = Outside {
            log: &mut |text| unsafe { log_l1(text, true) },
            // If no shutdown thread can be created there is nothing more to report: a report
            // is being made already, and the way out below ends the game anyway.
            quit: &mut || unsafe {
                let _ = request_quit();
            },
            thread_slot: &|| unsafe { find_thread_slot() as u32 },
        };
        match unsafe { report(&context, &mut outside) } {
            Ok(Leave::Return) => return_to_game as *const () as u32,
            Ok(Leave::Exit) => {
                // `find_thread_slot` found nothing: its null result is in esi.
                r.esi = 0;
                game_addr(EXIT_PATH_RVA) as u32
            }
            Ok(Leave::Routine { slot, routine }) => {
                r.esi = slot;
                routine
            }
            Err(_) => original,
        }
    };
    if !shadow() {
        log(&format!(
            "fatal::report: the Rust version ran and continues at {target:#x} (the original reporter is at {original:#x})"
        ));
    }
    // The slot the thunk reserved sits right above the saved registers and flags.
    unsafe { (r as *mut Regs as *mut u32).add(9).write(target) };
}

// ---- self-test: a copy of the original that cannot end the game ----

const FUNCTION_LEN: usize = 0x219;
const ENTRY_BYTES: [u8; 5] = [0x60, 0x9C, 0x5A, 0x8B, 0xEC]; // pusha; pushf; pop edx; mov ebp, esp
const CALL_OFFSETS: [usize; 25] = [
    64, 87, 162, 199, 213, 243, 255, 287, 298, 309, 320, 331, 342, 353, 364, 388, 405, 422, 439,
    456, 463, 468, 489, 500, 518,
];
const FIND_SLOT_CALL: usize = 523;
/// The operand of the `jz` to the exit path.
const EXIT_JUMP_OPERAND: usize = 530;

static LOGGED: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];
static LOG_COUNT: AtomicU32 = AtomicU32::new(0);
static QUITS: AtomicU32 = AtomicU32::new(0);
static LANDED: AtomicU32 = AtomicU32::new(0);
static FAKE_SLOT: AtomicU32 = AtomicU32::new(0); // what the stand-in for find_thread_slot returns
static FAKE_SLOT_BODY: AtomicU32 = AtomicU32::new(0); // a slot: its first dword is the routine
static ENTRY_FLAGS: AtomicU32 = AtomicU32::new(0);
static COPY: AtomicUsize = AtomicUsize::new(0);

/// Stands in for the log entry point: notes the text pointer in eax.
#[unsafe(naked)]
unsafe extern "C" fn fake_log() {
    core::arch::naked_asm!(
        "push ecx",
        "mov ecx, [{count}]",
        "and ecx, 3",
        "mov [{logged} + ecx * 4], eax",
        "inc dword ptr [{count}]",
        "pop ecx",
        "ret",
        count = sym LOG_COUNT,
        logged = sym LOGGED,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn fake_quit() {
    core::arch::naked_asm!("inc dword ptr [{quits}]", "ret", quits = sym QUITS)
}

/// Stands in for `find_thread_slot`: esi = the slot the test chose, zero flag if none.
#[unsafe(naked)]
unsafe extern "C" fn fake_find_thread_slot() {
    core::arch::naked_asm!("mov esi, [{slot}]", "test esi, esi", "ret", slot = sym FAKE_SLOT)
}

/// Where the copy's jump to the exit path lands: back in the test.
#[unsafe(naked)]
unsafe extern "C" fn land_exit() {
    core::arch::naked_asm!("mov dword ptr [{landed}], 1", "ret", landed = sym LANDED)
}

/// The "start routine" of the test's thread slot: back in the test.
#[unsafe(naked)]
unsafe extern "C" fn land_routine() {
    core::arch::naked_asm!("mov dword ptr [{landed}], 2", "ret", landed = sym LANDED)
}

/// Enters the copy the way the game enters the reporter: by a jump, with chosen flags.
#[unsafe(naked)]
unsafe extern "C" fn enter_copy() {
    core::arch::naked_asm!(
        "push dword ptr [{flags}]",
        "popfd",
        "jmp dword ptr [{copy}]",
        flags = sym ENTRY_FLAGS,
        copy = sym COPY,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let redirect = |target: usize| match target {
        t if t == rva(0x40142B) => Some(fake_log as *const () as usize),
        t if t == QUIT_RVA => Some(fake_quit as *const () as usize),
        t if t == rva(0x410CC5) => Some(fake_find_thread_slot as *const () as usize),
        _ => None,
    };
    let mut calls = CALL_OFFSETS.to_vec();
    calls.push(FIND_SLOT_CALL);
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x40763C)),
            FUNCTION_LEN,
            &ENTRY_BYTES,
            &calls,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.retarget(EXIT_JUMP_OPERAND, land_exit as *const () as usize) };
    COPY.store(copy.addr(), Ordering::SeqCst);
    FAKE_SLOT_BODY.store(land_routine as *const () as u32, Ordering::SeqCst);
    let fake_slot = &FAKE_SLOT_BODY as *const AtomicU32 as u32;

    // Everything the reporter reads or writes in the game, put back at the end. The
    // "reported" switch above all: left at 1 it makes the game show a message box on exit.
    let regions: [Region; 8] = [
        (game_addr(TEXT_RVA) as u32, TEXT_LEN),
        (game_addr(TEMPLATE_RVA) as u32, TEMPLATE.len() + 1),
        (game_addr(REPORTED_RVA) as u32, 1),
        (game_addr(LOG_ENABLED_RVA) as u32, 4),
        (game_addr(CODE_RVA) as u32, 8),
        (game_addr(EXTRA_RVA) as u32, 0x40),
        (game_addr(LAST_FILE_PATH_RVA) as u32, 4),
        (game_addr(W32_RVA) as u32, 4),
    ];
    let saved = capture(&regions);
    let written = &regions[..4];

    struct Case {
        code: u32,
        message: Option<&'static [u8]>,
        extra: &'static [u8],
        last_file: Option<&'static [u8]>,
        w32: u32,
        reported: u8,
        slot: bool,
        flags: u32,
    }
    let case = |code, message, slot| Case {
        code,
        message,
        extra: b"\0",
        last_file: None,
        w32: 0,
        reported: 0,
        slot,
        flags: 0x202,
    };
    let cases = [
        case(0x80A1_9D97, None, false),
        case(0x8070_DA00, Some(b"Invalid TrakPak Header\0"), true),
        case(0x0000_1234, Some(b"just a breakpoint\0"), false),
        case(0x0000_1234, Some(b"\0"), true),
        case(0x1000_0001, Some(b"text only\0"), false),
        case(0x9000_0002, Some(b"text only, fatal\0"), true),
        case(0x9000_0002, None, false),
        Case {
            extra: b"extra line\r\n\0",
            ..case(0x8000_0001, Some(b"one\\ntwo\\\\n\\x\0"), false)
        },
        Case {
            last_file: Some(b"var\\some.dat\0"),
            w32: 0xDEAD_BEEF,
            ..case(0x8000_0002, Some(b"with file\0"), true)
        },
        Case {
            flags: 0x202 | 0x8C5,
            ..case(0x0000_0003, None, false)
        },
        Case {
            flags: 0x202 | 0x41,
            w32: 5,
            ..case(0x8000_0004, Some(b"flags\0"), false)
        },
        Case {
            reported: 1,
            ..case(0x8000_0005, Some(b"second report\0"), true)
        },
        Case {
            reported: 1,
            ..case(0x0000_0006, Some(b"second breakpoint\0"), false)
        },
    ];

    for (n, c) in cases.iter().enumerate() {
        let prepare = || unsafe {
            restore(&regions, &saved);
            game_ptr::<u8>(TEXT_RVA).write_bytes(0xAA, TEXT_LEN);
            game_write(CODE_RVA, c.code);
            game_write(MESSAGE_RVA, c.message.map_or(0, |m| m.as_ptr() as u32));
            std::ptr::copy_nonoverlapping(c.extra.as_ptr(), game_ptr(EXTRA_RVA), c.extra.len());
            game_write(
                LAST_FILE_PATH_RVA,
                c.last_file.map_or(0, |p| p.as_ptr() as u32),
            );
            game_write(W32_RVA, c.w32);
            game_write(REPORTED_RVA, c.reported);
            game_write(LOG_ENABLED_RVA, 0u32);
            for counter in LOGGED.iter().chain([&LOG_COUNT, &QUITS, &LANDED]) {
                counter.store(0, Ordering::SeqCst);
            }
            FAKE_SLOT.store(if c.slot { fake_slot } else { 0 }, Ordering::SeqCst);
            ENTRY_FLAGS.store(c.flags, Ordering::SeqCst);
        };
        let observed = || {
            (
                capture(written),
                LOGGED.each_ref().map(|p| p.load(Ordering::SeqCst)),
                LOG_COUNT.load(Ordering::SeqCst),
                QUITS.load(Ordering::SeqCst),
            )
        };

        prepare();
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_game(&mut ro, enter_copy as *const () as usize) };
        let theirs = observed();
        let landed = LANDED.load(Ordering::SeqCst);
        // The stack pointer the original saw cannot be known beforehand: take the one it
        // printed, if it printed the registers.
        let text_o = unsafe { cstr(game_ptr(TEXT_RVA)) };
        let printed_esp =
            (c.reported == 0 && c.code & TEXT_ONLY == 0 && text_o.len() >= TEMPLATE.len())
                .then(|| {
                    let dump = &text_o[text_o.len() - TEMPLATE.len()..];
                    std::str::from_utf8(&dump[ESP_AT..ESP_AT + 8])
                        .ok()
                        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                })
                .flatten();

        prepare();
        let context = Context {
            eax: before.eax,
            ebx: before.ebx,
            ecx: before.ecx,
            edx: before.edx,
            esi: before.esi,
            edi: before.edi,
            esp: printed_esp.unwrap_or(0),
            ebp: before.ebp,
            flags: c.flags,
        };
        let mut logged = Vec::new();
        let mut quits = 0u32;
        let slot = FAKE_SLOT.load(Ordering::SeqCst);
        let leave = unsafe {
            report(
                &context,
                &mut Outside {
                    log: &mut |text| logged.push(text as u32),
                    quit: &mut || quits += 1,
                    thread_slot: &|| slot,
                },
            )
        };
        let mut logged_r = [0u32; 4];
        logged_r[..logged.len().min(4)].copy_from_slice(&logged[..logged.len().min(4)]);
        let mine = (capture(written), logged_r, logged.len() as u32, quits);

        // How the copy left, and with which registers: all as they were, except that after a
        // fatal code esi is what `find_thread_slot` returned: the slot, or 0.
        let (expected_landed, expected_esi) = match leave {
            Ok(Leave::Return) => (0, before.esi),
            Ok(Leave::Exit) => (1, 0),
            Ok(Leave::Routine { slot, routine }) if routine == land_routine as *const () as u32 => {
                (2, slot)
            }
            _ => (u32::MAX, 0),
        };
        let mut expected_regs = before;
        expected_regs.esi = expected_esi;
        let regs_ok = (ro.eax, ro.ebx, ro.ecx, ro.edx, ro.esi, ro.edi, ro.ebp)
            == (
                expected_regs.eax,
                expected_regs.ebx,
                expected_regs.ecx,
                expected_regs.edx,
                expected_regs.esi,
                expected_regs.edi,
                expected_regs.ebp,
            );
        let ok = theirs == mine && landed == expected_landed && regs_ok;
        t.check(ok, || {
            let text_r = unsafe { cstr(game_ptr(TEXT_RVA)) };
            format!(
                "case {n} code={:#x}: original landed={landed} logged={:x?} count={} quits={} esi={:#x}; rust {leave:x?} logged={:x?} count={} quits={} same_memory={} text={}",
                c.code,
                theirs.1,
                theirs.2,
                theirs.3,
                ro.esi,
                mine.1,
                mine.2,
                mine.3,
                theirs.0 == mine.0,
                bytes_str(&text_r[..text_r.len().min(300)])
            )
        });
    }

    restore(&regions, &saved);
    t.finish()
}
