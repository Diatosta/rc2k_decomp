//! The picture converters' table and the MMX ones.
//!
//! The table has a converter for each surface format number (seven); a format without one
//! is converted pixel by pixel. The MMX converters take four pixels of four 8-bit
//! components a step (two quadwords) and make four 16-bit pixels (one quadword), with the
//! masks of their constants (five quadwords each, the game's data).

/// What the table holds for a format number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Converter {
    /// None: pixel by pixel.
    None,
    /// Components to 4-4-4-4 pixels, row by row.
    Components4444,
    /// Rows copied as they are.
    CopyRows,
    /// The MMX converter of a format (1 .. 4).
    Mmx(u32),
    /// The 3DNow converter of a format (1 .. 4).
    ThreeDNow(u32),
}

pub const CONVERTERS: usize = 7;

/// The table for a processor with MMX (`has_mmx` 1) and 3DNow (`has_3dnow` 1) or not.
pub fn converters(has_mmx: u32, has_3dnow: u32) -> [Converter; CONVERTERS] {
    let mut table = [Converter::None; CONVERTERS];
    table[4] = Converter::Components4444;
    table[5] = Converter::CopyRows;
    table[6] = Converter::CopyRows;
    if has_mmx == 1 {
        for (n, entry) in table.iter_mut().enumerate().take(5).skip(1) {
            *entry = if has_3dnow == 1 {
                Converter::ThreeDNow(n as u32)
            } else {
                Converter::Mmx(n as u32)
            };
        }
    }
    table
}

// ---- MMX's operations on a quadword ----

fn words(x: u64) -> [u16; 4] {
    [
        x as u16,
        (x >> 16) as u16,
        (x >> 32) as u16,
        (x >> 48) as u16,
    ]
}

fn quad(w: [u16; 4]) -> u64 {
    w[0] as u64 | (w[1] as u64) << 16 | (w[2] as u64) << 32 | (w[3] as u64) << 48
}

fn psrlw(x: u64, n: u32) -> u64 {
    quad(words(x).map(|w| w >> n))
}

fn psllw(x: u64, n: u32) -> u64 {
    quad(words(x).map(|w| w << n))
}

/// `packuswb a, b`: the eight signed words to unsigned bytes, saturated; a's four first.
fn packuswb(a: u64, b: u64) -> u64 {
    let sat = |w: u16| (w as i16).clamp(0, 255) as u64;
    let (a, b) = (words(a), words(b));
    let mut out = 0u64;
    for (n, &w) in a.iter().chain(b.iter()).enumerate() {
        out |= sat(w) << (8 * n);
    }
    out
}

/// Four pixels (`q0`, `q1`) made four of format `format` (1 5-6-5, 2 5-5-5, 3 1-5-5-5,
/// else 4-4-4-4) with the converter's masks `k`.
pub fn step(format: u32, k: &[u64; 5], q0: u64, q1: u64) -> u64 {
    let (m4, m5, m6, m7) = (k[0], k[1], k[2], k[3]);
    let low = packuswb(q0 & m4, q1 & m4);
    let high = packuswb(psrlw(q0, 8) & m4, psrlw(q1, 8) & m4);
    match format {
        1 => (low & m5) | (psrlw(low, 3) & m7) | (psllw(high, 3) & m6),
        2 => (psrlw(low, 1) & m5) | (psrlw(low, 3) & m7) | (psllw(high, 2) & m6),
        3 => (psrlw(low, 1) & m5) | (high & k[4]) | (psrlw(low, 3) & m7) | (psllw(high, 2) & m6),
        _ => (psrlw(low, 4) & m5) | (high & k[4]) | (psrlw(low, 4) & m7) | (high & m6),
    }
}

/// Where the next row is written: `to` (past the row) moved by `gap`, forward for the
/// MMX 5-6-5 converter, back for the others (each converter's own way, kept).
pub fn next_row(to: u32, gap: u32, format: u32, three_dnow: bool) -> u32 {
    if format == 1 && !three_dnow {
        to.wrapping_add(gap)
    } else {
        to.wrapping_sub(gap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table() {
        use Converter::*;
        assert_eq!(
            converters(0, 1),
            [None, None, None, None, Components4444, CopyRows, CopyRows]
        );
        assert_eq!(converters(1, 0)[1..5], [Mmx(1), Mmx(2), Mmx(3), Mmx(4)]);
        assert_eq!(
            converters(1, 1)[1..5],
            [ThreeDNow(1), ThreeDNow(2), ThreeDNow(3), ThreeDNow(4)]
        );
    }

    #[test]
    fn pixels_565() {
        // The 5-6-5 masks: the components, red, green, blue.
        let k = [
            0x00FF_00FF_00FF_00FF,
            0xF800_F800_F800_F800,
            0x07E0_07E0_07E0_07E0,
            0x001F_001F_001F_001F,
            0,
        ];
        // white, red, green, blue (bytes b, g, r, a)
        let q0 = 0x00FF_0000_00FF_FFFF;
        let q1 = 0x0000_00FF_0000_FF00;
        assert_eq!(step(1, &k, q0, q1), 0x001F_07E0_F800_FFFF);
        assert_eq!(next_row(0x100, 8, 1, false), 0x108);
        assert_eq!(next_row(0x100, 8, 1, true), 0xF8);
    }
}
