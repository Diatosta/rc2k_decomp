//! The fatal-error report (converted 2026-10-21 from `fatal::report` 0x40763C and
//! `fatal::show_report_at_exit` 0x407855): its text, made once per run, and how the
//! reporter leaves.
//!
//! A code with bit 31 is fatal (the game quits), one without is a breakpoint (the game goes
//! on); bit 28 makes a report of the message alone.

use crate::strings::format::{binary, hex};

/// Bit 31 of a code: the game quits.
pub const FATAL: u32 = 0x8000_0000;
/// Bit 28: the message alone (no heading, no file, no registers), and the message box's
/// caption "Rally: Guru Meditation" cut to "Rally".
pub const TEXT_ONLY: u32 = 0x1000_0000;
/// The room for the report, its NUL included (`Text`, 0x9526F8).
pub const TEXT_LEN: usize = 0x400;
/// Where the caption is cut for a code with bit 28 (its byte 5 made 0).
pub const CAPTION_CUT: usize = 5;

/// The register lines' template (0x4F673A), which the original fills in place.
pub const TEMPLATE: &[u8] = b"EAX=XXXXXXXX\tEBX=XXXXXXXX\tECX=XXXXXXXX\tEDX=XXXXXXXX\r\nESI=XXXXXXXX\tEDI=XXXXXXXX\tESP=XXXXXXXX\tEBP=XXXXXXXX\r\nC=X  Z=X  S=X  V=X  P=X";
/// Where the eight register values go in the template.
const HEX_AT: [usize; 8] = [4, 17, 30, 43, 57, 70, 83, 96];
/// Where the five flags go, with their bits: carry, zero, sign, overflow, parity.
const FLAG_AT: [(usize, u32); 5] = [(108, 1), (113, 0x40), (118, 0x80), (123, 0x800), (128, 4)];

/// The registers and flags at the place that failed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

/// The register and flag lines: the template with the values in.
pub fn register_dump(c: &Context) -> Vec<u8> {
    let mut dump = TEMPLATE.to_vec();
    let values = [c.eax, c.ebx, c.ecx, c.edx, c.esi, c.edi, c.esp, c.ebp];
    for (at, value) in HEX_AT.into_iter().zip(values) {
        // Eight digits are always given.
        if let Some(digits) = hex(value, 8) {
            dump[at..at + 8].copy_from_slice(&digits);
        }
    }
    for (at, bit) in FLAG_AT {
        if let Some(digit) = binary((c.flags & bit != 0) as u32, 1) {
            dump[at] = digit[0];
        }
    }
    dump
}

/// The report, without its NUL: unless the code has bit 28, "Error code [XXXXXXXX]" (bit
/// 31) or "Breakpoint [XXXXXXXX]"; the message (an empty one as none) with the two
/// characters `\n` made a line break; `extra` (the string at 0x4F67BD); unless bit 28,
/// "Last file: `path'", "W32=XXXXXXXX" when `w32` is not 0, and the register lines.
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
        text.extend(hex(code, 8).unwrap_or_default());
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
            text.extend(hex(w32, 8).unwrap_or_default());
            text.extend_from_slice(b"\r\n");
        }
        text.extend(register_dump(context));
    }
    text
}

/// Whether a report of `len` bytes fits its room with its NUL (one that does not is not
/// worked out: the original writes past `Text`).
pub fn fits(len: usize) -> bool {
    len < TEXT_LEN
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

/// The way out once the report is made: for a fatal code `quit` is asked first, then the
/// current thread's slot (0: none) and, for one, its routine (the slot's +0).
pub fn leave(
    code: u32,
    quit: &mut dyn FnMut(),
    slot: &dyn Fn() -> u32,
    routine: &dyn Fn(u32) -> u32,
) -> Leave {
    if code & FATAL == 0 {
        return Leave::Return;
    }
    quit();
    match slot() {
        0 => Leave::Exit,
        slot => Leave::Routine {
            slot,
            routine: routine(slot),
        },
    }
}

/// `fatal::show_report_at_exit`: whether the box's caption is cut to "Rally" (byte
/// `CAPTION_CUT` made 0) for this code.
pub fn caption_cut(code: u32) -> bool {
    code & TEXT_ONLY != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts() {
        let c = Context {
            eax: 0x1234_5678,
            esp: 0xDEAD_BEEF,
            flags: 0x41,
            ..Context::default()
        };
        let dump = register_dump(&c);
        assert_eq!(&dump[..12], b"EAX=12345678");
        assert_eq!(&dump[83..91], b"DEADBEEF");
        assert_eq!(&dump[106..], b"C=1  Z=1  S=0  V=0  P=0");
        let t = report_text(
            0x8010_8571,
            Some(b"Need\\nmore"),
            b"",
            Some(b"a.txt"),
            5,
            &c,
        );
        let head =
            b"Error code [80108571]\r\nNeed\r\nmore\r\nLast file: `a.txt'\r\nW32=00000005\r\n";
        assert_eq!(&t[..head.len()], head);
        assert_eq!(&t[head.len()..], &dump[..]);
        let t = report_text(0x1000_0001, Some(b""), b"x\r\n", Some(b"a"), 5, &c);
        assert_eq!(t, b"x\r\n");
        let t = report_text(0x0000_0002, Some(b"\\"), b"", None, 0, &c);
        assert!(t.starts_with(b"Breakpoint [00000002]\r\n\\\r\nEAX="));
        assert!(fits(0x3FF) && !fits(0x400));
    }

    #[test]
    fn ways_out() {
        let mut asked = 0;
        assert_eq!(
            leave(1, &mut || asked += 1, &|| 5, &|s| s + 1),
            Leave::Return
        );
        assert_eq!(asked, 0);
        assert_eq!(
            leave(FATAL, &mut || asked += 1, &|| 0, &|s| s + 1),
            Leave::Exit
        );
        assert_eq!(
            leave(FATAL | 3, &mut || asked += 1, &|| 5, &|s| s + 1),
            Leave::Routine {
                slot: 5,
                routine: 6
            }
        );
        assert_eq!(asked, 2);
        assert!(caption_cut(TEXT_ONLY) && !caption_cut(FATAL));
    }
}
