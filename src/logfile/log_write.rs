//! `logfile::log_write` (0x4013CB): the entry of the string log function: checks that logging
//! is on, takes the log's lock and goes on into `write_line`.
//!
//! In:  eax = text, dl = 1 for a line break after it, dh = level - 1
//! Out: as `write_line` leaves the registers if it got that far. Nothing changes if
//!      `g_logEnabled` is 0 or less. If `g_logLevel` is 0, only eax changes, to the code
//!      address 0x4013D9: the original's lock function then returns straight to the caller
//!      of this one, with its own return address popped into eax.
//! The four entries `log_write_l1`, `_l1_nl`, `_l2`, `_l2_nl` (0x401427 ..) set edx and jump
//! here, so they go through this hook too, as do the four `log_newline`/`log_space` helpers.
//! The lock is `g_logLock`, taken with an `xchg` and `Sleep(1)` between tries
//! (`log_lock_acquire`, 0x40130A, inlined here); `write_line` releases it.
//! It writes the real log file, so shadow mode runs the original alone. The thunk does not
//! wait for self-tests and the self-test runs with the game running, like `write_line`'s.

use crate::framework::{
    EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write,
    hook_asm, rva, shadow,
};
use crate::logfile::write_line::{
    LINE_BUF_RVA, LOG_INDENT_RVA, LOG_LEVEL_RVA, LOG_LINE_START_RVA, acquire, file_bytes, file_len,
    format_line, lock, write_line_regs,
};
use rc2k_core::files::log::{Entry, entry};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::log_write", rva(0x4013CB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    log_lock
);

// ---- game data ----

const LOG_ENABLED_RVA: usize = rva(0x4E0094); // g_logEnabled
/// What eax holds after a call with `g_logLevel` 0: the address after the original's call
/// to its lock function.
const SKIPPED_EAX_RVA: usize = rva(0x4013D9);

// ---- Rust version ----

/// `log_write` with the original's registers in and out.
pub unsafe fn log_write_regs(r: &mut Regs) {
    unsafe {
        // Converted: `rc2k_core::files::log::entry`.
        match entry(game_read::<i32>(LOG_ENABLED_RVA), || {
            game_read::<u32>(LOG_LEVEL_RVA)
        }) {
            Entry::Off => return,
            Entry::Skipped => {
                r.eax = game_addr(SKIPPED_EAX_RVA) as u32;
                return;
            }
            Entry::Written => {}
        }
        acquire();
        write_line_regs(r);
    }
}

/// For Rust callers: logs a NUL-terminated text at `level` (1 or 2), with a line break after
/// it if `newline`.
pub unsafe fn log_write(text: *const u8, newline: bool, level: u8) {
    let mut regs = Regs {
        eax: text as u32,
        edx: (level.wrapping_sub(1) as u32) << 8 | newline as u32,
        ..Regs::default()
    };
    unsafe { log_write_regs(&mut regs) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { log_write_regs(r) };
}

// ---- self-test: appends lines to the real var\logfile.txt ----

type Impl = unsafe fn(&mut Regs);
unsafe fn original_regs(r: &mut Regs) {
    unsafe { call_original(r) }
}

/// Sets the line state under the lock, then runs `f` with the lock free, as a caller would.
/// Returns the registers before and after, and the file length before.
unsafe fn run(
    f: Impl,
    msg: *const u8,
    dh: u8,
    dl: u8,
    start: u32,
    indent: i32,
) -> (Regs, Regs, u64) {
    acquire();
    unsafe {
        game_write(LOG_LINE_START_RVA, start);
        game_write(LOG_INDENT_RVA, indent as u32);
    }
    let before = file_len();
    lock().store(0, Ordering::Release);
    let given = Regs {
        eax: msg as u32,
        edx: (dh as u32) << 8 | dl as u32,
        ..Regs::SENTINEL
    };
    let mut r = given;
    unsafe { f(&mut r) };
    (given, r, before)
}

fn appended(before: u64, wanted: &[u8]) -> bool {
    wanted.is_empty()
        || file_bytes(before, 1 << 16)
            .windows(wanted.len())
            .any(|w| w == wanted)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let impls: [(&str, Impl); 2] = [("original", original_regs), ("rust", log_write_regs)];
    let saved_enabled: i32 = unsafe { game_read(LOG_ENABLED_RVA) };
    let level: u32 = unsafe { game_read(LOG_LEVEL_RVA) };
    acquire();
    let (saved_start, saved_indent): (u32, u32) =
        unsafe { (game_read(LOG_LINE_START_RVA), game_read(LOG_INDENT_RVA)) };
    lock().store(0, Ordering::Release);
    let buf = game_ptr::<u8>(LINE_BUF_RVA) as u32;

    // Logged lines, if the game's log level lets anything through.
    if level != 0 {
        let dh = (level as u8).wrapping_sub(1);
        unsafe { game_write(LOG_ENABLED_RVA, 1i32) };
        for (msg, newline, indent, start) in [
            (&b"rc2k_hook selftest: log_write\0"[..], true, 0, 0u32),
            (
                &b"rc2k_hook selftest: log_write, no newline\0"[..],
                false,
                0,
                0,
            ),
            (
                &b"rc2k_hook selftest: log_write, indented\0"[..],
                true,
                2,
                1,
            ),
        ] {
            let text = &msg[..msg.len() - 1];
            let (want, _) = format_line(text, newline, indent, start);
            for (who, f) in impls {
                let (given, r, before) =
                    unsafe { run(f, msg.as_ptr(), dh, newline as u8, start, indent) };
                let ok = appended(before, &want)
                    && r.eax == want.len() as u32
                    && r.ebx == buf
                    && r.ecx == want.len() as u32
                    && r.edx == newline as u32
                    && r.preserved_except(&given, EAX | EBX | ECX | EDX);
                t.check(ok, || {
                    format!(
                        "{who} {:?}: eax={:#x} ebx={:#x} ecx={} edx={}",
                        String::from_utf8_lossy(text),
                        r.eax,
                        r.ebx,
                        r.ecx,
                        r.edx
                    )
                });
            }
        }
        // The level gate inside the tail: one level too high is not written, and the lock is
        // free again afterwards (the next case would hang otherwise).
        if level < 255 {
            let msg = b"rc2k_hook selftest: log_write MUST NOT APPEAR\0";
            for (who, f) in impls {
                let (given, r, before) = unsafe { run(f, msg.as_ptr(), level as u8, 1, 0, 0) };
                let ok = !appended(before, &msg[..msg.len() - 1])
                    && r.eax == msg.as_ptr() as u32
                    && r.ecx == level + 1
                    && r.edx == 1
                    && r.preserved_except(&given, ECX | EDX);
                t.check(ok, || {
                    format!("{who} level gate: eax={:#x} ecx={}", r.eax, r.ecx)
                });
            }
        }
    }

    // Logging switched off: nothing is written and nothing changes.
    let msg = b"rc2k_hook selftest: log_write WHILE OFF\0";
    for enabled in [0i32, -1] {
        unsafe { game_write(LOG_ENABLED_RVA, enabled) };
        for (who, f) in impls {
            let (given, r, before) = unsafe { run(f, msg.as_ptr(), 0, 1, 0, 0) };
            let ok = !appended(before, &msg[..msg.len() - 1]) && r.preserved_except(&given, 0);
            t.check(ok, || format!("{who} enabled={enabled}: eax={:#x}", r.eax));
        }
    }

    // Log level 0: the lock function returns to the caller's caller, leaving a code address in
    // eax. Other threads' messages are dropped while the level is 0, which is a moment.
    unsafe { game_write(LOG_ENABLED_RVA, 1i32) };
    for (who, f) in impls {
        unsafe { game_write(LOG_LEVEL_RVA, 0u32) };
        let (given, r, before) = unsafe { run(f, msg.as_ptr(), 0, 1, 0, 0) };
        unsafe { game_write(LOG_LEVEL_RVA, level) };
        let ok = !appended(before, &msg[..msg.len() - 1])
            && r.eax == game_addr(SKIPPED_EAX_RVA) as u32
            && r.preserved_except(&given, EAX);
        t.check(ok, || format!("{who} level 0: eax={:#x}", r.eax));
    }

    unsafe { game_write(LOG_ENABLED_RVA, saved_enabled) };
    acquire();
    unsafe {
        game_write(LOG_LINE_START_RVA, saved_start);
        game_write(LOG_INDENT_RVA, saved_indent);
    }
    lock().store(0, Ordering::Release);
    t.finish()
}
