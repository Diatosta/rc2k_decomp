//! The fatal-error reporter.

use crate::framework::GameHook;

pub mod raise;
pub mod report;
pub mod show_report_at_exit;

pub static HOOKS: &[&dyn GameHook] = &[&report::HOOK];

/// The reporter's variables, out of the image since 2026-10-24 (phase 5; each has its own
/// entry in `moved::MOVED`).
pub mod state {
    use crate::moved::Place;
    use std::sync::atomic::{AtomicU8, AtomicU32};

    /// g_fatalCode (0x4F52A4): the code of the error being reported.
    pub static CODE: AtomicU32 = AtomicU32::new(0);
    /// g_fatalMessage (0x4F52A8): the pointer of its message, or 0.
    pub static MESSAGE: AtomicU32 = AtomicU32::new(0);
    /// dword_4E0598 (0x4E0598): the Win32 error to name in a report, or 0 (nothing in the
    /// program sets it).
    pub static W32: AtomicU32 = AtomicU32::new(0);
    /// g_fatalReported (0x4F67BC, a byte): 1 once a report was written.
    pub static REPORTED: AtomicU8 = AtomicU8::new(0);
    /// `cursor` in IDA (0x4F67BD, 128 bytes): a string added to every report (empty).
    pub const EXTRA_LEN: usize = 128;
    pub static EXTRA: Place<[u8; EXTRA_LEN]> = Place::new([0; EXTRA_LEN]);
    /// aEaxXxxxxxxxEbx (0x4F673A, 130 bytes): the registers' template, filled in place.
    pub const TEMPLATE_LEN: usize = 130;
    pub static TEMPLATE: Place<[u8; TEMPLATE_LEN]> = Place::new(template());
    /// Text (0x9526F8, 0x400 bytes, bss): the report.
    pub const TEXT_LEN: usize = 0x400;
    pub static TEXT: Place<[u8; TEXT_LEN]> = Place::new([0; TEXT_LEN]);
    /// Caption (0x4F66FD): "Rally: Guru Meditation", of which the log shows the part from
    /// +7 ("aGuruMeditation + 1"); byte 5 (byte_4F6702) is cut to 0 by the report's end.
    pub const CAPTION_LEN: usize = 23;
    pub static CAPTION: Place<[u8; CAPTION_LEN]> = Place::new(*b"Rally: Guru Meditation\0");

    const fn template() -> [u8; TEMPLATE_LEN] {
        let text = b"EAX=XXXXXXXX\tEBX=XXXXXXXX\tECX=XXXXXXXX\tEDX=XXXXXXXX\r\nESI=XXXXXXXX\tEDI=XXXXXXXX\tESP=XXXXXXXX\tEBP=XXXXXXXX\r\nC=X  Z=X  S=X  V=X  P=X";
        let mut out = [0u8; TEMPLATE_LEN];
        let mut n = 0;
        while n < text.len() {
            out[n] = text[n];
            n += 1;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::state::*;
    use crate::moved::image_bytes;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn state_starts_as_the_games() {
        let word = |ida| u32::from_le_bytes(image_bytes(ida, 4).try_into().unwrap());
        assert_eq!(word(0x4F52A4), CODE.load(Relaxed));
        assert_eq!(word(0x4F52A8), MESSAGE.load(Relaxed));
        assert_eq!(word(0x4E0598), W32.load(Relaxed));
        assert_eq!(image_bytes(0x4F67BC, 1)[0], REPORTED.load(Relaxed));
        let bytes = |p: *const u8, n| unsafe { std::slice::from_raw_parts(p, n).to_vec() };
        assert_eq!(
            bytes(EXTRA.ptr().cast(), EXTRA_LEN),
            image_bytes(0x4F67BD, EXTRA_LEN)
        );
        assert_eq!(
            bytes(TEMPLATE.ptr().cast(), TEMPLATE_LEN),
            image_bytes(0x4F673A, TEMPLATE_LEN)
        );
        assert_eq!(
            bytes(TEXT.ptr().cast(), TEXT_LEN),
            image_bytes(0x9526F8, TEXT_LEN)
        );
        assert_eq!(
            bytes(CAPTION.ptr().cast(), CAPTION_LEN),
            image_bytes(0x4F66FD, CAPTION_LEN)
        );
    }
}
