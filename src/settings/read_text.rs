//! `settings::read_text` (0x41155A): reads a settings file into the scratch block.
//!
//! In:  eax = path
//! Out: read: esi = the text, carry clear (zero set); not read: carry set, esi unchanged;
//!      everything else preserved
//! `files::read_file` of up to 0xFFFF bytes to 0x93E8B0, and a NUL after what was read.
//! Five callers: `var\ral.cfg`, `var\ral.zog`, `lang.cfg`, `files\d\gfx\sku` and one more.
//! Shadow mode reads twice and compares the text.

use crate::errors::BadInput;
use crate::files::read_file::read_file;
use crate::files::stream::LastPath;
use crate::framework::{ESI, RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::settings::{TEXT_ROOM, TEXT_RVA};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("settings::read_text", rva(0x41155A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The text (NUL-terminated) and its length, or None if the file was not read.
/// Refuses what `read_file` refuses.
pub unsafe fn read_text(path: *const u8) -> Result<Option<(*const u8, u32)>, BadInput> {
    unsafe {
        let text = game_addr(TEXT_RVA) as *mut u8;
        let read = read_file(path, text, TEXT_ROOM, 0)?;
        // Converted: `rc2k_core::files::settings::text_read`.
        let Some(bytes) = rc2k_core::files::settings::text_read(read.ok, read.bytes_read) else {
            return Ok(None);
        };
        text.add(bytes as usize).write(0);
        Ok(Some((text, bytes)))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match unsafe { read_text(r.eax as *const u8) } {
        Ok(mine) => mine,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        if let Some((text, _)) = mine {
            r.esi = text as u32;
        }
        r.set_carry(mine.is_none());
        r.set_zero(mine.is_some());
        return;
    }
    let text_of = |read: Option<(*const u8, u32)>| {
        read.map(|(text, bytes)| {
            unsafe { std::slice::from_raw_parts(text, bytes as usize + 1) }.to_vec()
        })
    };
    let my_text = text_of(mine);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let their_text = text_of((!o.carry()).then_some((o.esi as *const u8, mine.map_or(0, |m| m.1))));
    if their_text != my_text || (o.carry() && o.esi != r.esi) || !o.preserved_except(r, ESI) {
        HOOK.mismatch(|| {
            format!(
                "path={:?}: original carry={} esi={:#x}; rust read {:?} bytes; same text={}",
                unsafe { std::ffi::CStr::from_ptr(r.eax as *const _) },
                o.carry(),
                o.esi,
                mine.map(|m| m.1),
                their_text == my_text
            )
        });
    }
    r.esi = o.esi;
    r.eflags = o.eflags;
}

// ---- self-test: a file of its own in the game's `var` folder ----

const TEST_FILE: &str = "var\\rc2k_hook_settings.cfg";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_settings.cfg\0";
const MISSING_PATH: &[u8] = b"var\\rc2k_hook_no_settings.cfg\0";
const TEST_TEXT: &[u8] = b"; a settings file\r\nfirst=1\r\nsecond = two\r\n";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if std::fs::write(TEST_FILE, TEST_TEXT).is_err() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let text = game_addr(TEXT_RVA) as *mut u8;
    let room = TEST_TEXT.len() + 0x10;
    let saved = unsafe { std::slice::from_raw_parts(text, room) }.to_vec();
    let last = LastPath::save();
    for path in [TEST_PATH, MISSING_PATH] {
        let before = Regs {
            eax: path.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let run = |rust: bool| unsafe {
            std::ptr::write_bytes(text, 0xAA, room);
            let result = if rust {
                read_text(path.as_ptr()).ok().map(|read| {
                    (
                        read.map_or(before.esi, |(text, _)| text as u32),
                        read.is_none(),
                    )
                })
            } else {
                let mut ro = before;
                call_original(&mut ro);
                ro.preserved_except(&before, ESI)
                    .then_some((ro.esi, ro.carry()))
            };
            (
                result,
                std::slice::from_raw_parts(text, room).to_vec(),
                LastPath::now(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            format!(
                "path={:?}: original {:x?} text {:?} last={:#x}; rust {:x?} text {:?} last={:#x}",
                String::from_utf8_lossy(path),
                theirs.0,
                crate::strings::utils::bytes_str(&theirs.1),
                theirs.2,
                mine.0,
                crate::strings::utils::bytes_str(&mine.1),
                mine.2
            )
        });
    }
    unsafe { std::ptr::copy(saved.as_ptr(), text, room) };
    last.restore();
    let _ = std::fs::remove_file(TEST_FILE);
    t.finish()
}
