//! `text::draw_time_text` (0x402395): draws a time's text one character at a time, each
//! with the font's fixed width (+0x1C) set to 6 for ':' and '.', 0x10 for the others, so
//! that the digits keep their columns; at most 11 characters, up to the NUL.
//!
//! In:  eax = font, ebx = text, ecx = x, edx = y
//! Out: every register kept (`pusha` .. `popa`); the flags of the loop's last test (no
//!      caller reads them)
//!
//! Each character is drawn as a text of its own: the dword after it is swapped for 0
//! (`xchg edi, [ebx+1]`, edi 0) around `text::draw_text_keep_regs`, then put back; x moves
//! on by the width. The font's +0x1C is 0 at the end. Callers: `net::lobby_frame` (twice).
//! Its call through `part` (`net::lobby_calls`: the Rust version). Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose call goes to a stand-in that notes
//! the character drawn and the font's width, against the Rust version, on texts of 0, 8,
//! 11 and 14 characters.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{DRAW_KEEP, Part, answer, dispatch_lobby, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::draw_time_text", rva(0x402395), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The font's fixed width.
const WIDTH: u32 = 0x1C;

fn read32(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

fn write32(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn draw_time_text(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edi = 0;
    r.ebp = 11;
    // Converted: `rc2k_core::text::time_text_widths`; here the original's way of drawing
    // one character at a time (the dword after it swapped for 0 around the call).
    let text = unsafe { crate::bridge::Raw::new(r.ebx as *const u8) };
    for width in crate::bridge::read_through(rc2k_core::text::time_text_widths(&text)) {
        let x = r.ecx;
        r.esi = width;
        write32(r.eax + WIDTH, r.esi);
        let kept = read32(r.ebx + 1);
        write32(r.ebx + 1, r.edi);
        r.edi = kept;
        part(DRAW_KEEP, &mut r)?;
        let zero = read32(r.ebx + 1);
        write32(r.ebx + 1, r.edi);
        r.edi = zero;
        r.ecx = x.wrapping_add(r.esi);
        r.ebx = r.ebx.wrapping_add(1);
        r.ebp = r.ebp.wrapping_sub(1);
    }
    write32(r.eax + WIDTH, 0);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, draw_time_text);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; xor edi, edi; mov ebp, 0Bh
    let copy = match crate::net::lobby::copy(
        0x402395,
        0x40,
        &[0x60, 0x33, 0xFF, 0xBD, 0x0B, 0x00, 0x00, 0x00],
        &[0x28],
        &[DRAW_KEEP],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut font = vec![0x5Au8; 0x20];
    let mut text = vec![0u8; 0x20];
    // What each draw saw: the six bytes from the character on, the font's width.
    let mut seen = vec![0u8; 16 * 10];
    let (f, x, s) = (
        font.as_mut_ptr() as u32,
        text.as_mut_ptr() as u32,
        seen.as_mut_ptr() as u32,
    );
    let regions = [(f, 0x20), (x, 0x20), (s, 16 * 10)];
    for sample in [
        &b""[..],
        b"01:23.45",
        b"1:02:03.04",
        b"12:34:56.78:9",
        b"1234",
    ] {
        run_both(
            &mut t,
            &format!("{:?}", String::from_utf8_lossy(sample)),
            copy.addr(),
            Regs {
                eax: f,
                ebx: x,
                ecx: 0x100,
                edx: 0x40,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &[],
            &|| unsafe {
                std::ptr::write_bytes(f as *mut u8, 0x5A, 0x20);
                std::ptr::write_bytes(x as *mut u8, 0xA5, 0x20);
                std::ptr::copy_nonoverlapping(sample.as_ptr(), x as *mut u8, sample.len());
                (x as *mut u8).add(sample.len()).write(0);
                std::ptr::write_bytes(s as *mut u8, 0, 16 * 10);
                let mut n = 0u32;
                answer(Box::new(move |_, r| {
                    if n < 16 {
                        let to = (s + 10 * n) as *mut u8;
                        std::ptr::copy_nonoverlapping(r.ebx as *const u8, to, 6);
                        (to.add(6) as *mut u32).write_unaligned(read32(r.eax + WIDTH));
                    }
                    n += 1;
                    r.set_carry(n & 1 == 0);
                }));
            },
            &draw_time_text,
        );
    }
    let _ = (&mut font, &mut text, &mut seen);
    t.finish()
}
