//! `logfile::date_text` (0x401D74): writes the date and time the game started at.
//!
//! In:  ebx = cursor
//! Out: ebx = the cursor after the text; everything else preserved
//! "Monday 12th October 2026 14:49:56.46": the day's name, the day with its ending, the
//! month's name, the year, the time with hundredths. From what `sub_410F29` took from
//! `GetLocalTime` at the game's start: `byte_51839F` the day of the week, `dword_5183A0`
//! the day, the month and (a word) the year, `dword_5183A4` hour, minute, second and
//! hundredths. After the text a carriage return, a line feed and two NULs are written,
//! which the cursor is not moved past. One caller, the log's start.

use crate::framework::{EBX, RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::service::harness::{capture, restore};
use crate::strings::format_u32::format_u32;
use crate::strings::str_append_indexed::str_append_indexed;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::date_text", rva(0x401D74), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const WEEKDAY_RVA: usize = rva(0x51839F); // byte_51839F, then day, month, year (word),
const DAY_RVA: usize = rva(0x5183A0); // hour, minute, second, hundredths
const MONTH_RVA: usize = rva(0x5183A1);
const YEAR_RVA: usize = rva(0x5183A2);
const TIME_RVA: usize = rva(0x5183A4);
const STARTED_LEN: usize = 9;
const WEEKDAYS_RVA: usize = rva(0x401EA5); // "Sunday" ..
const ENDING_OF_DAY_RVA: usize = rva(0x401E23); // byte_401E23: per day, which of
const ENDINGS_RVA: usize = rva(0x401E43); // "st", "nd", "rd", "th"
const MONTHS_RVA: usize = rva(0x401E4E); // "", "January" ..

// ---- Rust version ----

/// The text, without what is written after it.
pub unsafe fn date_text() -> Vec<u8> {
    unsafe {
        let mut text = vec![0u8; 0x200];
        let start = text.as_mut_ptr();
        let name = |table: usize, index: u8, cursor: *mut u8| {
            str_append_indexed(game_addr(table) as *const u8, index as i32, cursor)
        };
        let put = |bytes: &[u8], cursor: *mut u8| {
            std::ptr::copy(bytes.as_ptr(), cursor, bytes.len());
            cursor.add(bytes.len())
        };
        let day: u8 = game_read(DAY_RVA);
        let mut cursor = name(WEEKDAYS_RVA, game_read(WEEKDAY_RVA), start);
        cursor = put(b" ", cursor);
        cursor = put(&format_u32(day as u32, 0), cursor);
        let ending: u8 = game_read(ENDING_OF_DAY_RVA + day as usize);
        cursor = name(ENDINGS_RVA, ending, cursor);
        cursor = put(b" ", cursor);
        cursor = name(MONTHS_RVA, game_read(MONTH_RVA), cursor);
        cursor = put(b" ", cursor);
        cursor = put(&format_u32(game_read::<u16>(YEAR_RVA) as u32, 0), cursor);
        cursor = put(b" ", cursor);
        let [hour, minute, second, hundredths]: [u8; 4] = game_read(TIME_RVA);
        for (part, after) in [
            (hour, &b":"[..]),
            (minute, b":"),
            (second, b"."),
            (hundredths, b""),
        ] {
            cursor = put(&format_u32(part as u32, 0x12), cursor);
            cursor = put(after, cursor);
        }
        text.truncate(cursor.offset_from(start) as usize);
        text
    }
}

/// For Rust callers: the text at the cursor, and after it CR, LF and two NULs. Returns the
/// cursor after the text.
pub unsafe fn write_date_text(cursor: *mut u8) -> *mut u8 {
    unsafe {
        let text = date_text();
        std::ptr::copy(text.as_ptr(), cursor, text.len());
        let end = cursor.add(text.len());
        (end as *mut u32).write_unaligned(0x0A0D);
        end
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        r.ebx = unsafe { write_date_text(r.ebx as *mut u8) } as u32;
        return;
    }
    let mut mine = unsafe { date_text() };
    mine.extend(0x0A0Du32.to_le_bytes());
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let written = (o.ebx.wrapping_sub(r.ebx) as usize).wrapping_add(4);
    let same = written == mine.len()
        && unsafe { std::slice::from_raw_parts(r.ebx as *const u8, written) } == &mine[..];
    if !same || !o.preserved_except(r, EBX) {
        HOOK.mismatch(|| {
            format!(
                "original wrote {written} bytes; rust {:?}",
                String::from_utf8_lossy(&mine)
            )
        });
    }
    r.ebx = o.ebx;
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let started = [(game_addr(WEEKDAY_RVA) as u32, STARTED_LEN)];
    let saved = capture(&started);
    // weekday, day, month, year (two bytes), hour, minute, second, hundredths
    let mut cases: Vec<[u8; STARTED_LEN]> = (1..=31u8)
        .map(|day| {
            [
                day % 7,
                day,
                day % 12 + 1,
                0xD0,
                0x07,
                day % 24,
                day,
                59 - day,
                day * 3,
            ]
        })
        .collect();
    cases.extend([
        [0, 1, 1, 0x00, 0x00, 0, 0, 0, 0],
        [6, 31, 12, 0xFF, 0xFF, 23, 59, 59, 99],
        [3, 12, 10, 0xEA, 0x07, 14, 49, 56, 46],
        [0, 0, 0, 0x0F, 0x27, 9, 9, 9, 9],
    ]);
    for case in cases {
        let run = |rust: bool| {
            restore(&started, &case);
            let mut out = [0xAAu8; 0x60];
            let before = Regs {
                ebx: out.as_mut_ptr() as u32 + 4,
                ..Regs::SENTINEL
            };
            let end = if rust {
                Some(unsafe { write_date_text(out.as_mut_ptr().add(4)) } as u32)
            } else {
                let mut ro = before;
                unsafe { call_original(&mut ro) };
                ro.preserved_except(&before, EBX).then_some(ro.ebx)
            }
            .map(|end| end.wrapping_sub(out.as_ptr() as u32 + 4));
            (end, out)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            format!(
                "started {case:x?}: original {:?} {:?}; rust {:?} {:?}",
                theirs.0,
                crate::strings::utils::bytes_str(&theirs.1),
                mine.0,
                crate::strings::utils::bytes_str(&mine.1)
            )
        });
    }
    restore(&started, &saved);
    t.finish()
}
