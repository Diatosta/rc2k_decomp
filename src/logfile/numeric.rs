//! What the six numeric log functions share. Each does what `log_write` does, with one step
//! in between: under the log's lock it formats the number in eax as text in the scratch
//! buffer `dst` (0x93E030), ends it with a NUL, and goes on into `write_line` with eax = the
//! scratch buffer and edx as given.
//!
//! Registers: as `log_write`. If the tail's own level gate skips the line, eax is the
//! scratch buffer and ebx the byte after the text's NUL. With `g_logLevel` 0 only eax
//! changes, to the address after that function's call to the lock function.
//! Each of the six has four entries before it that set edx (`_l1`, `_l1_nl`, `_l2`,
//! `_l2_nl`) and jump to it, so they go through its hook.

use crate::errors::BadInput;
use crate::framework::rva;
use crate::framework::{
    EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, shadow,
};
use crate::logfile::write_line::{
    LINE_BUF_RVA, LOG_INDENT_RVA, LOG_LEVEL_RVA, LOG_LINE_START_RVA, acquire, file_bytes, file_len,
    format_line, lock, write_line_regs,
};
use crate::screens::harness::CallOriginal;
use std::sync::atomic::Ordering;

const LOG_ENABLED_RVA: usize = rva(0x4E0094); // g_logEnabled
const SCRATCH_RVA: usize = rva(0x93E030); // dst: 0x50 bytes below the log's line buffer
/// The longest text accepted: the scratch buffer with room for the NUL.
const SCRATCH_TEXT: usize = 0x4F;

fn scratch() -> *mut u8 {
    game_ptr(SCRATCH_RVA)
}

/// The text of a number, as it is left in the scratch buffer.
pub struct NumberText {
    /// The bytes written, from the start of the buffer.
    pub bytes: Vec<u8>,
    /// Where the NUL goes. `log_float` puts it where the wanted number of decimals ends,
    /// which can be before or after the end of `bytes`.
    pub nul_at: usize,
}

impl NumberText {
    pub fn plain(bytes: Vec<u8>) -> Self {
        let nul_at = bytes.len();
        Self { bytes, nul_at }
    }
}

/// One of the six functions.
pub struct Body {
    /// The address after its call to the lock function: what eax holds if `g_logLevel` is 0.
    pub after_lock_call_rva: usize,
    /// The text for the registers at entry (the number is in eax, some take ecx too).
    pub text: fn(&Regs) -> Result<NumberText, BadInput>,
}

/// The function with the original's registers in and out. Refuses a text that does not fit
/// the scratch buffer, before anything is done.
pub unsafe fn log_number_regs(body: &Body, r: &mut Regs) -> Result<(), BadInput> {
    unsafe {
        if game_read::<i32>(LOG_ENABLED_RVA) <= 0 {
            return Ok(());
        }
        if game_read::<u32>(LOG_LEVEL_RVA) == 0 {
            r.eax = game_addr(body.after_lock_call_rva) as u32;
            return Ok(());
        }
        let text = (body.text)(r)?;
        if text.bytes.len() > SCRATCH_TEXT || text.nul_at > SCRATCH_TEXT {
            return Err(BadInput(
                "the number's text does not fit the scratch buffer",
            ));
        }
        acquire();
        std::ptr::copy_nonoverlapping(text.bytes.as_ptr(), scratch(), text.bytes.len());
        scratch().add(text.nul_at).write(0);
        r.eax = scratch() as u32;
        r.ebx = scratch() as u32 + text.nul_at as u32 + 1;
        write_line_regs(r);
    }
    Ok(())
}

/// For Rust callers.
pub unsafe fn log_number(body: &Body, eax: u32, ecx: u32, newline: bool, level: u8) {
    let mut regs = Regs {
        eax,
        ecx,
        edx: (level.wrapping_sub(1) as u32) << 8 | newline as u32,
        ..Regs::default()
    };
    // A text too long for the buffer is not logged at all.
    let _ = unsafe { log_number_regs(body, &mut regs) };
}

/// The whole `dispatch` of such a hook. It writes the real log, so shadow mode runs the
/// original alone.
pub fn dispatch_number(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, body: &Body) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { log_number_regs(body, r) } {
        unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
    }
}

// ---- self-test: appends lines to the real var\logfile.txt ----

/// Runs one version with the line state set, the scratch buffer filled with 'z' and the lock
/// free. Returns the registers before and after, the file length before and the scratch
/// buffer afterwards.
unsafe fn run(f: &dyn Fn(&mut Regs), eax: u32, ecx: u32, edx: u32) -> (Regs, Regs, u64, Vec<u8>) {
    acquire();
    unsafe {
        game_write(LOG_LINE_START_RVA, 1u32);
        game_write(LOG_INDENT_RVA, 0u32);
        scratch().write_bytes(b'z', SCRATCH_TEXT + 1);
    }
    let before = file_len();
    lock().store(0, Ordering::Release);
    let given = Regs {
        eax,
        ecx,
        edx,
        ..Regs::SENTINEL
    };
    let mut r = given;
    f(&mut r);
    // Another thread logging a number right now would disturb this; it is rare.
    acquire();
    let left = unsafe { std::slice::from_raw_parts(scratch(), SCRATCH_TEXT + 1) }.to_vec();
    lock().store(0, Ordering::Release);
    (given, r, before, left)
}

/// One of the two versions a test runs: its name for the failure line, and how to call it.
type Version<'a> = (&'a str, &'a dyn Fn(&mut Regs));

/// `inputs` are (eax, ecx) pairs. Every one is logged by the original and by the Rust
/// version; what each leaves in the scratch buffer, in the file and in the registers must be
/// what the Rust text function says. Then the paths that log nothing.
pub fn check_number(
    t: &mut SelfTest,
    call_original: CallOriginal,
    body: &Body,
    inputs: &[(u32, u32)],
) {
    let original = |r: &mut Regs| unsafe { call_original(r) };
    let rust = |r: &mut Regs| {
        let _ = unsafe { log_number_regs(body, r) };
    };
    let versions: [Version; 2] = [("original", &original), ("rust", &rust)];
    let saved_enabled: i32 = unsafe { game_read(LOG_ENABLED_RVA) };
    let level: u32 = unsafe { game_read(LOG_LEVEL_RVA) };
    acquire();
    let saved_line: (u32, u32) =
        unsafe { (game_read(LOG_LINE_START_RVA), game_read(LOG_INDENT_RVA)) };
    lock().store(0, Ordering::Release);
    let line_buffer = game_ptr::<u8>(LINE_BUF_RVA) as u32;
    let scratch_addr = scratch() as u32;

    unsafe { game_write(LOG_ENABLED_RVA, 1i32) };
    for &(eax, ecx) in inputs {
        let given = Regs {
            eax,
            ecx,
            ..Regs::SENTINEL
        };
        let Ok(text) = (body.text)(&given) else {
            continue; // refused: the original is left alone with it
        };
        if text.bytes.len() > SCRATCH_TEXT || text.nul_at > SCRATCH_TEXT {
            continue;
        }
        // What the scratch buffer must hold: the text over the 'z' filling, with its NUL.
        let mut wanted = vec![b'z'; SCRATCH_TEXT + 1];
        wanted[..text.bytes.len()].copy_from_slice(&text.bytes);
        wanted[text.nul_at] = 0;
        let logged = &wanted[..text.nul_at];

        if level != 0 {
            let dh = (level as u8).wrapping_sub(1) as u32;
            let (line, _) = format_line(logged, true, 0, 1);
            for (who, f) in versions {
                let (given, r, before, left) = unsafe { run(f, eax, ecx, dh << 8 | 1) };
                let in_file = file_bytes(before, 1 << 16)
                    .windows(line.len())
                    .any(|w| w == line);
                let ok = left == wanted
                    && in_file
                    && r.eax == line.len() as u32
                    && r.ebx == line_buffer
                    && r.ecx == line.len() as u32
                    && r.edx == 1
                    && r.preserved_except(&given, EAX | EBX | ECX | EDX);
                t.check(ok, || {
                    format!(
                        "{who} eax={eax:#x} ecx={ecx:#x}: wanted {:?}, scratch {:?}, in_file={in_file} eax={:#x} ebx={:#x} ecx={} edx={}",
                        String::from_utf8_lossy(logged),
                        String::from_utf8_lossy(&left[..left.iter().position(|&b| b == 0).unwrap_or(left.len())]),
                        r.eax,
                        r.ebx,
                        r.ecx,
                        r.edx
                    )
                });
            }
            // One level too high: formatted, then dropped by the tail.
            if level < 255 {
                for (who, f) in versions {
                    let (given, r, _, left) = unsafe { run(f, eax, ecx, level << 8 | 1) };
                    let ok = left == wanted
                        && r.eax == scratch_addr
                        && r.ebx == scratch_addr + text.nul_at as u32 + 1
                        && r.ecx == level + 1
                        && r.edx == 1
                        && r.preserved_except(&given, EAX | EBX | ECX | EDX);
                    t.check(ok, || {
                        format!(
                            "{who} eax={eax:#x} ecx={ecx:#x}, level gate: eax={:#x} ebx={:#x} ecx={}",
                            r.eax, r.ebx, r.ecx
                        )
                    });
                }
            }
        }
    }

    // Logging off: nothing changes. Log level 0: only eax, to the address after the call.
    let (eax, ecx) = inputs.first().copied().unwrap_or((1, 0));
    for (who, f) in versions {
        unsafe { game_write(LOG_ENABLED_RVA, 0i32) };
        let (given, r, _, left) = unsafe { run(f, eax, ecx, 1) };
        unsafe { game_write(LOG_ENABLED_RVA, 1i32) };
        t.check(
            r.preserved_except(&given, 0) && left.iter().all(|&b| b == b'z'),
            || format!("{who} logging off: eax={:#x}", r.eax),
        );

        unsafe { game_write(LOG_LEVEL_RVA, 0u32) };
        let (given, r, _, left) = unsafe { run(f, eax, ecx, 1) };
        unsafe { game_write(LOG_LEVEL_RVA, level) };
        let ok = r.eax == game_addr(body.after_lock_call_rva) as u32
            && r.preserved_except(&given, EAX)
            && left.iter().all(|&b| b == b'z');
        t.check(ok, || format!("{who} level 0: eax={:#x}", r.eax));
    }

    unsafe { game_write(LOG_ENABLED_RVA, saved_enabled) };
    acquire();
    unsafe {
        game_write(LOG_LINE_START_RVA, saved_line.0);
        game_write(LOG_INDENT_RVA, saved_line.1);
    }
    lock().store(0, Ordering::Release);
}
