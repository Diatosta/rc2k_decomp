//! `strings::append_time` (0x401A39): appends a time given in hundredths of a second, as
//! "MM:SS.cc" or "HH:MM:SS.cc".
//!
//! In:  eax = hundredths of a second, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! The hours field is written from one hour upwards, or always when `g_timeShowHours` is 1.

use crate::framework::{
    EBX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use crate::strings::format_u32::format_u32;
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::append_time", rva(0x401A39), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SHOW_HOURS_RVA: usize = rva(0x4E0148); // g_timeShowHours: 1 = always write the hours

fn show_hours() -> bool {
    unsafe { game_read::<u32>(SHOW_HOURS_RVA) == 1 }
}

// ---- Rust version ----

const HOUR: u32 = 360_000;
const MINUTE: u32 = 6_000;
const SECOND: u32 = 100;
const TWO_DIGITS: u8 = 0x12; // format_u32: zero-padded, width 2

pub fn append_time(hundredths: u32, show_hours: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = hundredths;
    if show_hours || rest >= HOUR {
        out.extend(format_u32(rest / HOUR, TWO_DIGITS));
        out.push(b':');
        rest %= HOUR;
    }
    out.extend(format_u32(rest / MINUTE, TWO_DIGITS));
    out.push(b':');
    rest %= MINUTE;
    out.extend(format_u32(rest / SECOND, TWO_DIGITS));
    out.push(b'.');
    out.extend(format_u32(rest % SECOND, TWO_DIGITS));
    out
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (hundredths, cursor) = (r.eax, r.ebx as *mut u8);
    let mine = append_time(hundredths, show_hours());

    if !shadow() {
        unsafe { copy_nonoverlapping(mine.as_ptr(), cursor, mine.len()) };
        r.ebx = (cursor as usize + mine.len()) as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine.len() || unsafe { from_raw_parts(cursor, orig_len) } != &mine[..] {
        HOOK.mismatch(|| {
            format!(
                "hundredths={hundredths}: original wrote {orig_len} bytes, rust {}",
                mine.len()
            )
        });
    }
    r.ebx = o.ebx;
}

// ---- self-test: both settings of g_timeShowHours ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(SHOW_HOURS_RVA) };

    for flag in [0u32, 1] {
        unsafe { game_write(SHOW_HOURS_RVA, flag) };
        let values = (0..=12_500u32).chain([
            HOUR - 1,
            HOUR,
            HOUR + 1,
            2 * HOUR + 123_456,
            100 * HOUR - 1,
            100 * HOUR,
            2_147_483_648,
            4_294_967_295,
        ]);

        for v in values {
            let mut buf = [0xAAu8; 32];
            let before = Regs {
                eax: v,
                ebx: buf.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
            let mine = append_time(v, flag == 1);
            let preserved = ro.preserved_except(&before, EBX);

            t.check(buf.get(..orig_len) == Some(&mine[..]) && preserved, || {
                format!(
                    "hundredths={v} show_hours={flag} original {} rust {} preserved={preserved}",
                    bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                    bytes_str(&mine)
                )
            });
        }
    }

    unsafe { game_write(SHOW_HOURS_RVA, saved) };
    t.finish()
}
