//! The fatal-error report: its text, made once per run, and how the reporter leaves.
//!
//! A code with bit 31 is fatal (the game quits); one without is a breakpoint (the game goes
//! on); bit 28 makes a report of the message alone.

use crate::strings::format::{binary, hex};

/// Bit 31 of a code: the game quits.
pub const FATAL: u32 = 0x8000_0000;
/// Bit 28: the message alone (no heading, no file, no registers), and the message box's
/// caption "Rally: Guru Meditation" cut to "Rally".
pub const TEXT_ONLY: u32 = 0x1000_0000;
/// The room for the report, its NUL included.
pub const TEXT_LEN: usize = 0x400;
/// Where the caption is cut for a code with bit 28 (its byte 5 made 0).
pub const CAPTION_CUT: usize = 5;

/// The register lines' template.
const TEMPLATE: &[u8] = b"EAX=XXXXXXXX\tEBX=XXXXXXXX\tECX=XXXXXXXX\tEDX=XXXXXXXX\r\nESI=XXXXXXXX\tEDI=XXXXXXXX\tESP=XXXXXXXX\tEBP=XXXXXXXX\r\nC=X  Z=X  S=X  V=X  P=X";
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
fn register_dump(c: &Context) -> Vec<u8> {
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

/// The report, without its NUL: unless bit 28, "Error code \[XXXXXXXX\]" (bit 31) or
/// "Breakpoint \[XXXXXXXX\]"; the message (empty as none) with the two characters `\n` made a
/// line break; unless bit 28, "Last file: `path'" and the register lines (left out without a
/// `context`).
pub fn report_text(
    code: u32,
    message: Option<&[u8]>,
    last_file: Option<&[u8]>,
    context: Option<&Context>,
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
    if full {
        if let Some(path) = last_file {
            text.extend_from_slice(b"Last file: `");
            text.extend_from_slice(path);
            text.extend_from_slice(b"'\r\n");
        }
        if let Some(context) = context {
            text.extend(register_dump(context));
        }
    }
    text
}

/// Whether a report of `len` bytes fits its room with its NUL (a longer one is refused).
pub fn fits(len: usize) -> bool {
    len < TEXT_LEN
}

/// How the reporter leaves; `T` names one of the game's own threads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leave<T> {
    /// Back to the game: the code was not fatal.
    Return,
    /// The program ends: a fatal code on a thread that is none of the game's own.
    Exit,
    /// The routine of the game's own thread the report was made on is run again.
    Routine(T),
}

/// The way out once the report is made: for a fatal code `quit` is asked first, then which of
/// the game's own threads this is (`own_thread`, None for any other).
pub fn leave<T>(code: u32, quit: &mut dyn FnMut(), own_thread: &dyn Fn() -> Option<T>) -> Leave<T> {
    if code & FATAL == 0 {
        return Leave::Return;
    }
    quit();
    match own_thread() {
        None => Leave::Exit,
        Some(thread) => Leave::Routine(thread),
    }
}

/// `fatal::show_report_at_exit`: whether the box's caption is cut to "Rally" (byte
/// `CAPTION_CUT` made 0) for this code.
pub fn caption_cut(code: u32) -> bool {
    code & TEXT_ONLY != 0
}

/// `text` cut into lines no wider than `max` by `width` (a word is not split unless it is
/// wider than a line alone, and then by characters); line breaks in the text start new lines.
pub fn wrap_lines(text: &str, max: i32, width: &dyn Fn(&str) -> i32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let mut word = word.to_string();
            // A word wider than a line alone is cut at the last character that fits.
            while width(&word) > max && word.chars().count() > 1 {
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                let mut cut = word.chars().count() - 1;
                while cut > 1 && width(&word.chars().take(cut).collect::<String>()) > max {
                    cut -= 1;
                }
                lines.push(word.chars().take(cut).collect());
                word = word.chars().skip(cut).collect();
            }
            let joined = if line.is_empty() {
                word.clone()
            } else {
                format!("{line} {word}")
            };
            if width(&joined) <= max || line.is_empty() {
                line = joined;
            } else {
                lines.push(std::mem::replace(&mut line, word));
            }
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_wrapped_to_the_width() {
        let width = |s: &str| s.chars().count() as i32;
        assert_eq!(
            wrap_lines("one two three four", 9, &width),
            vec!["one two", "three", "four"]
        );
        assert_eq!(wrap_lines("a\nb", 9, &width), vec!["a", "b"]);
        assert_eq!(
            wrap_lines("abcdefghij", 4, &width),
            vec!["abcd", "efgh", "ij"]
        );
        assert_eq!(wrap_lines("", 4, &width), vec![""]);
    }

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
        let t = report_text(0x8010_8571, Some(b"Need\\nmore"), Some(b"a.txt"), Some(&c));
        let head = b"Error code [80108571]\r\nNeed\r\nmore\r\nLast file: `a.txt'\r\n";
        assert_eq!(&t[..head.len()], head);
        assert_eq!(&t[head.len()..], &dump[..]);
        let t = report_text(0x1000_0001, Some(b"x"), Some(b"a"), Some(&c));
        assert_eq!(t, b"x\r\n");
        let t = report_text(0x1000_0001, Some(b""), Some(b"a"), Some(&c));
        assert_eq!(t, b"");
        let t = report_text(0x0000_0002, Some(b"\\"), None, Some(&c));
        assert!(t.starts_with(b"Breakpoint [00000002]\r\n\\\r\nEAX="));
        let t = report_text(0x0000_0002, Some(b"\\"), None, None);
        assert_eq!(t, b"Breakpoint [00000002]\r\n\\\r\n");
        assert!(fits(0x3FF) && !fits(0x400));
    }

    #[test]
    fn ways_out() {
        let mut asked = 0;
        assert_eq!(leave(1, &mut || asked += 1, &|| Some(5)), Leave::Return);
        assert_eq!(asked, 0);
        assert_eq!(
            leave::<u32>(FATAL, &mut || asked += 1, &|| None),
            Leave::Exit
        );
        assert_eq!(
            leave(FATAL | 3, &mut || asked += 1, &|| Some(5)),
            Leave::Routine(5)
        );
        assert_eq!(asked, 2);
        assert!(caption_cut(TEXT_ONLY) && !caption_cut(FATAL));
    }
}
